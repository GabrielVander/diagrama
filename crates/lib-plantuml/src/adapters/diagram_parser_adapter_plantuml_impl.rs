use std::collections::HashMap;

use async_trait::async_trait;
use lib_core::domain::{
    adapters::diagram_parser_adapter::DiagramParserAdapter,
    entities::diagram::{
        ArrowType, Cluster, ClusterType, Diagram, DiagramKind, Edge, EdgeStyle, Element,
        InteractionType, LineType, Node, NodeType, Note, NotePosition,
    },
};
use pest::{
    Parser,
    iterators::{Pair, Pairs},
};

use crate::infra::pest::plantuml_pest_parser::{PlantumlPestParser, Rule};

pub struct DiagramParserAdapterPlantumlImpl;

impl DiagramParserAdapterPlantumlImpl {
    fn parse_statement(&self, pair: Pair<Rule>) -> Option<Element> {
        match pair.as_rule() {
            Rule::class_def => Some(self.map_class(pair)),
            Rule::relation_def => Some(self.map_relation(pair)),
            Rule::package_def => Some(self.map_package(pair)),
            Rule::note_def => Some(self.map_note(pair)),
            _ => None, // Ignore skinparams/hides for now
        }
    }

    fn map_class(&self, pair: Pair<Rule>) -> Element {
        let mut inner: Pairs<Rule> = pair.into_inner();

        let kind_str: &str = inner.next().unwrap().as_str();
        let node_type: NodeType = match kind_str {
            "interface" => NodeType::Interface,
            "actor" => NodeType::Actor,
            "database" => NodeType::Database,
            _ => NodeType::Class,
        };

        let id: String = inner.next().unwrap().as_str().replace("\"", "");

        let mut label: Option<String> = Some(id.clone());
        let mut properties: HashMap<String, String> = HashMap::new();

        for part in inner {
            match part.as_rule() {
                Rule::alias => {
                    let alias: &str = part.into_inner().next().unwrap().as_str();
                    // If there is an alias, usually the ID stays internal, label becomes the quoted name
                    // But for simplicity here:
                    label = Some(alias.to_string());
                }
                Rule::stereotype => {
                    properties.insert("stereotype".to_string(), part.as_str().to_string());
                }
                Rule::body_block => {
                    // We could parse methods/fields here and put them in properties
                    // e.g. properties.insert("members", part.as_str())
                }
                _ => {}
            }
        }

        Element::Node(Node {
            id,
            label,
            node_type,
            properties,
        })
    }

    fn map_relation(&self, pair: Pair<Rule>) -> Element {
        let mut inner: Pairs<Rule> = pair.into_inner();

        let left_id: String = inner.next().unwrap().as_str().replace("\"", "");
        let arrow_str: &str = inner.next().unwrap().as_str();
        let right_id: String = inner.next().unwrap().as_str().replace("\"", "");

        let label = inner.next().map(|p| self.clean_label(p.as_str()));

        // Basic heuristic to determine arrow type from string
        // A real implementation needs a more robust arrow parser
        let (interaction, style): (InteractionType, EdgeStyle) = self.parse_arrow_string(arrow_str);

        Element::Edge(Edge {
            from: left_id,
            to: right_id,
            label,
            interaction,
            style,
        })
    }

    fn map_package(&self, pair: Pair<Rule>) -> Element {
        let mut inner: Pairs<Rule> = pair.into_inner();
        let _kind: &str = inner.next().unwrap().as_str(); // "package"
        let id: String = inner.next().unwrap().as_str().replace("\"", "");

        let mut children: Vec<Element> = Vec::new();

        // Find the block and parse inner statements recursively
        for part in inner {
            if part.as_rule() == Rule::statement {
                let inner_stmt: Pair<'_, Rule> = part.into_inner().next().unwrap();
                if let Some(child) = self.parse_statement(inner_stmt) {
                    children.push(child);
                }
            }
        }

        Element::Cluster(Cluster {
            id: id.clone(),
            label: Some(id),
            cluster_type: ClusterType::Package,
            children,
            properties: HashMap::new(),
        })
    }

    fn map_note(&self, pair: Pair<Rule>) -> Element {
        // Simplified mapping for "note right of X: text"
        let str_repr: &str = pair.as_str();
        Element::Note(Note {
            id: format!("note_{}", str_repr.len()), // generate increasing ID
            text: str_repr.to_string(),
            position: NotePosition::Floating,
            target_node_id: None,
        })
    }

    fn parse_arrow_string(&self, arrow: &str) -> (InteractionType, EdgeStyle) {
        let line: LineType = if arrow.contains("..") {
            LineType::Dotted
        } else {
            LineType::Solid
        };

        let interaction: InteractionType = if arrow.contains("|>") {
            InteractionType::Inheritance
        } else if arrow.contains("*") {
            InteractionType::Composition
        } else {
            InteractionType::Association
        };

        // TODO: strictly parse head/tail
        let style: EdgeStyle = EdgeStyle {
            line,
            head: ArrowType::Vee,
            tail: ArrowType::None,
        };

        (interaction, style)
    }

    fn clean_label(&self, s: &str) -> String {
        s.trim()
            .trim_matches(':')
            .trim()
            .trim_matches('"')
            .to_string()
    }
}

#[async_trait]
impl DiagramParserAdapter for DiagramParserAdapterPlantumlImpl {
    async fn parse(&self, source: &str) -> Result<Diagram, String> {
        let mut pairs: Pairs<Rule> = PlantumlPestParser::parse(Rule::file, source)
            .map_err(|e| format!("Parse error: {}", e))?;

        let root_pair: Pair<Rule> = pairs.next().ok_or("Empty input")?;

        let mut elements: Vec<Element> = Vec::new();

        for statement in root_pair.into_inner() {
            match statement.as_rule() {
                Rule::statement => {
                    // Extract the inner specific rule (class_def, relation_def, etc.)
                    let inner: Pair<Rule> = statement.into_inner().next().unwrap();
                    if let Some(element) = self.parse_statement(inner) {
                        elements.push(element);
                    }
                }
                Rule::EOI => break,
                _ => {} // Skip start_uml/end_uml checks inside the loop
            }
        }

        Ok(Diagram {
            title: None,              // Could extract from 'title' keyword if added
            kind: DiagramKind::Class, // Defaulting for now
            elements,
            styles: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use lib_core::domain::{
        adapters::diagram_parser_adapter::DiagramParserAdapter,
        entities::diagram::{Diagram, Element},
    };
    use pretty_assertions::assert_eq;

    use crate::adapters::diagram_parser_adapter_plantuml_impl::DiagramParserAdapterPlantumlImpl;

    #[test]
    fn test_parse_packages() {
        smol::block_on(async {
            let input: &str = r#"
            package "Core" {
                class Service
            }
            "#;
            let parser: DiagramParserAdapterPlantumlImpl = DiagramParserAdapterPlantumlImpl;

            let diagram: Diagram = parser.parse(input).await.expect("Failed to parse package");

            match &diagram.elements[0] {
                Element::Cluster(c) => {
                    assert_eq!(c.id, "Core");
                    assert_eq!(c.children.len(), 1); // Contains Service
                }
                _ => panic!("Expected Cluster"),
            }
        });
    }
}
