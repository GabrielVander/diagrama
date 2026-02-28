use std::collections::HashMap;

use async_trait::async_trait;
use lib_core::domain::{
    adapters::graph_parser::{FrontendError, GraphParser},
    entities::graph::Graph,
};

use crate::infra::{
    parser::{self, PlantUmlParseError},
    transformer,
};

pub struct GraphParserPlantumlImpl;

impl GraphParserPlantumlImpl {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GraphParser for GraphParserPlantumlImpl {
    async fn parse(&self, source: &str) -> Result<Graph, FrontendError> {
        parser::parse_plantuml(source)
            .map_err(FrontendError::from)
            .map(|ast| transformer::GraphBuilder::new().build(ast))
    }
}

impl From<PlantUmlParseError> for FrontendError {
    fn from(err: PlantUmlParseError) -> Self {
        match err {
            PlantUmlParseError::Syntax {
                message,
                line,
                column,
            } => FrontendError::Parse {
                source: "plantuml".into(),
                message,
                line,
                column,
            },
            PlantUmlParseError::Internal(msg) => FrontendError::Semantic {
                source: "plantuml".into(),
                message: msg,
            },
            PlantUmlParseError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => FrontendError::Parse {
                source: "plantuml".into(),
                message: format!("Unexpected token {}, expected {}", found, expected),
                line,
                column,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_core::domain::{
        adapters::graph_parser::GraphParser,
        entities::{
            edge::{Edge, EdgeKind},
            group::Group,
            node::{Node, NodeKind},
        },
    };

    #[test]
    fn test_frontend_error_from_plantuml_syntax_error() {
        let plantuml_err: PlantUmlParseError = PlantUmlParseError::Syntax {
            message: "Missing bracket".to_string(),
            line: 42,
            column: 12,
        };

        let frontend_err: FrontendError = plantuml_err.into();

        match frontend_err {
            FrontendError::Parse {
                source,
                message,
                line,
                column,
            } => {
                assert_eq!(source, "plantuml");
                assert_eq!(message, "Missing bracket");
                assert_eq!(line, 42);
                assert_eq!(column, 12);
            }
            _ => panic!("Expected FrontendError::Parse, got a different variant"),
        }
    }

    #[test]
    fn test_frontend_error_from_plantuml_internal_error() {
        let plantuml_err: PlantUmlParseError =
            PlantUmlParseError::Internal("Out of memory".to_string());

        let frontend_err: FrontendError = plantuml_err.into();

        match frontend_err {
            FrontendError::Semantic { source, message } => {
                assert_eq!(source, "plantuml");
                assert_eq!(message, "Out of memory");
            }
            _ => panic!("Expected FrontendError::Semantic, got a different variant"),
        }
    }

    #[test]
    fn test_frontend_error_from_plantuml_unexpected_token() {
        let plantuml_err: PlantUmlParseError = PlantUmlParseError::UnexpectedToken {
            expected: "-->".to_string(),
            found: "-X-".to_string(),
            line: 5,
            column: 20,
        };

        let frontend_err: FrontendError = plantuml_err.into();

        match frontend_err {
            FrontendError::Parse {
                source,
                message,
                line,
                column,
            } => {
                assert_eq!(source, "plantuml");
                assert_eq!(message, "Unexpected token -X-, expected -->");
                assert_eq!(line, 5);
                assert_eq!(column, 20);
            }
            _ => panic!("Expected FrontendError::Parse, got a different variant"),
        }
    }

    #[test]
    fn test_parse_black_box_wiring() {
        smol::block_on(async {
            let parser: GraphParserPlantumlImpl = GraphParserPlantumlImpl::new();

            // We use a basic PlantUML string. We aren't testing the actual AST building here
            // (that belongs in transformer.rs and parser.rs tests), just that the async
            // boundary and result mapping in GraphParserPlantumlImpl::parse are wired correctly.
            let valid_source: &str = "@startuml\nclass A\n@enduml";
            let invalid_source: &str = "INVALID_SYNTAX_12345";

            let valid_result: Result<Graph, FrontendError> = parser.parse(valid_source).await;
            let invalid_result: Result<Graph, FrontendError> = parser.parse(invalid_source).await;

            // We expect the valid source to at least not panic and return a parsed graph
            assert!(
                valid_result.is_ok(),
                "Expected Ok for valid source, got error: {:?}",
                valid_result.err()
            );

            // We expect garbage input to safely propagate an error through the From implementation.
            assert!(
                invalid_result.is_err(),
                "Expected Err for invalid source, but got Ok"
            );
        })
    }

    #[test]
    fn test_parse_basic_nodes_and_relations() {
        smol::block_on(async {
            let parser: GraphParserPlantumlImpl = GraphParserPlantumlImpl::new();
            let source: &str = r#"
            @startuml
            class "Customer" as C
            database "OrdersDB" as DB
            
            C --> DB : "places order"
            @enduml
            "#;

            let graph: Graph = parser
                .parse(source)
                .await
                .expect("Failed to parse valid PlantUML");

            assert_eq!(graph.nodes.len(), 2, "Should have exactly 2 nodes");
            assert_eq!(graph.edges.len(), 1, "Should have exactly 1 edge");

            let customer_node: &Node =
                find_node_by_label(&graph, "Customer").expect("Missing Customer node");
            assert_eq!(customer_node.kind, NodeKind::Entity);

            let db_node: &Node =
                find_node_by_label(&graph, "OrdersDB").expect("Missing OrdersDB node");
            assert_eq!(db_node.kind, NodeKind::Database);

            let edge: &Edge = find_edge_between_labels(&graph, "Customer", "OrdersDB")
                .expect("Missing edge between Customer and OrdersDB");

            assert_eq!(edge.kind, EdgeKind::Association);
            assert!(edge.directed, "Edge should be directed");
            assert_eq!(edge.label.as_deref(), Some("places order"));
        });
    }

    #[test]
    fn test_parse_groups_and_nesting() {
        smol::block_on(async {
            let parser: GraphParserPlantumlImpl = GraphParserPlantumlImpl::new();
            let source: &'static str = r#"
            @startuml
            package "Backend System" {
                component "API Gateway"
                component "Auth Service"
            }
            @enduml
            "#;

            let graph: Graph = parser
                .parse(source)
                .await
                .expect("Failed to parse group PlantUML");

            assert_eq!(graph.nodes.len(), 2, "Should have exactly 2 nodes");
            assert_eq!(graph.groups.len(), 1, "Should have exactly 1 group");

            let group: &Group = find_group_by_label(&graph, "Backend System")
                .expect("Missing Backend System group");

            let api_node: &Node =
                find_node_by_label(&graph, "API Gateway").expect("Missing API Gateway node");
            let auth_node: &Node =
                find_node_by_label(&graph, "Auth Service").expect("Missing Auth node");

            assert!(
                group.children.contains(&api_node.id),
                "Group missing API Gateway child"
            );
            assert!(
                group.children.contains(&auth_node.id),
                "Group missing Auth Service child"
            );

            assert_eq!(api_node.parent.as_ref(), Some(&group.id));
            assert_eq!(auth_node.parent.as_ref(), Some(&group.id));
        });
    }

    #[test]
    fn test_implicit_node_creation_from_relations() {
        smol::block_on(async {
            let parser: GraphParserPlantumlImpl = GraphParserPlantumlImpl::new();
            // Here, A and B are never explicitly defined with "class" or "component",
            // they only appear in a relation. The transformer should create them implicitly.
            let source: &str = r#"
            @startuml
            Client --* Server
            @enduml
        "#;

            let graph: Graph = parser
                .parse(source)
                .await
                .expect("Failed to parse implicit relation PlantUML");

            assert_eq!(
                graph.nodes.len(),
                2,
                "Should have implicitly created 2 nodes"
            );

            let client_node: &Node = find_node_by_label(&graph, "Client")
                .or_else(|| {
                    // If the label wasn't set for implicit nodes, it might just use the ID as the label.
                    graph.nodes.values().find(|n: &&Node| n.id == "Client")
                })
                .expect("Missing Client node");

            let server_node: &Node = find_node_by_label(&graph, "Server")
                .or_else(|| graph.nodes.values().find(|n: &&Node| n.id == "Server"))
                .expect("Missing Server node");

            let edge: &Edge = graph.edges.values().next().expect("Missing edge");
            assert_eq!(edge.from, client_node.id);
            assert_eq!(edge.to, server_node.id);
            assert_eq!(
                edge.kind,
                EdgeKind::Composition,
                "Expected composition for --*"
            );
        });
    }

    fn find_node_by_label<'a>(graph: &'a Graph, label: &str) -> Option<&'a Node> {
        graph
            .nodes
            .values()
            .find(|n: &&Node| n.label.as_deref() == Some(label))
    }

    fn find_edge_between_labels<'a>(
        graph: &'a Graph,
        from_label: &str,
        to_label: &str,
    ) -> Option<&'a Edge> {
        let from_node: &Node = find_node_by_label(graph, from_label)?;
        let to_node: &Node = find_node_by_label(graph, to_label)?;

        graph
            .edges
            .values()
            .find(|e: &&Edge| e.from == from_node.id && e.to == to_node.id)
    }

    fn find_group_by_label<'a>(graph: &'a Graph, label: &str) -> Option<&'a Group> {
        graph
            .groups
            .values()
            .find(|g: &&Group| g.label.as_deref() == Some(label))
    }
}
