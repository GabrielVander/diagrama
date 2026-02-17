use std::collections::HashMap;

use lib_core::domain::entities::diagram::{
    ArrowType, Cluster, ClusterType, Diagram, DiagramKind, Edge, EdgeStyle, Element,
    InteractionType, LineType, Node, NodeType, Note, NotePosition,
};

use crate::infra::ast::plant_uml_ast::{
    PlantUmlAst, UmlArrowEnd, UmlElement, UmlElementKind, UmlLineStyle, UmlNote, UmlNotePosition,
    UmlPackage, UmlPackageKind, UmlRelation, UmlStatement,
};

pub struct AstToDiagramMapper {
    note_counter: usize,
}

impl AstToDiagramMapper {
    pub fn new() -> Self {
        Self { note_counter: 0 }
    }

    pub fn map(&mut self, ast: PlantUmlAst) -> Diagram {
        let kind: DiagramKind = Self::determine_diagram_kind(&ast.statements);

        let elements: Vec<Element> = ast
            .statements
            .into_iter()
            .map(|stmt| self.map_statement(stmt))
            .collect();

        Diagram {
            title: ast.header.and_then(|h| h.title),
            kind,
            elements,
            styles: HashMap::new(),
        }
    }

    /// Recursively scans the AST statements to deduce the appropriate DiagramKind.
    /// This removes the hardcoded temporary implementation and paves the way
    /// for Flowchart, State, and Sequence diagrams as the pest grammar expands.
    fn determine_diagram_kind(statements: &[UmlStatement]) -> DiagramKind {
        for stmt in statements {
            match stmt {
                UmlStatement::Element(elem) => {
                    match elem.kind {
                        // Explicit structural keywords strongly indicate a Class Diagram
                        UmlElementKind::Class
                        | UmlElementKind::Interface
                        | UmlElementKind::AbstractClass
                        | UmlElementKind::Enum => return DiagramKind::Class,

                        // Actors and databases could technically belong to sequence or component diagrams,
                        // but without sequence-specific messages, we continue scanning.
                        UmlElementKind::Actor
                        | UmlElementKind::Database
                        | UmlElementKind::Component => continue,
                    }
                }
                UmlStatement::Package(pkg) => {
                    // Recursively inspect package contents
                    let child_kind = Self::determine_diagram_kind(&pkg.children);
                    // If a child explicitly determines the type, bubble it up.
                    return child_kind;
                }
                // If we encounter notes or relations, we keep looking for definitive elements.
                UmlStatement::Note(_) | UmlStatement::Relation(_) => {
                    continue;
                }
            }
        }

        // Fallback: If no definitive sequence, state, or flowchart markers are found,
        // the current AST syntax naturally models a structural/class diagram.
        DiagramKind::Class
    }

    fn map_statement(&mut self, stmt: UmlStatement) -> Element {
        match stmt {
            UmlStatement::Element(elem) => Element::Node(self.map_element(elem)),
            UmlStatement::Relation(rel) => Element::Edge(self.map_relation(rel)),
            UmlStatement::Package(pkg) => Element::Cluster(self.map_package(pkg)),
            UmlStatement::Note(note) => Element::Note(self.map_note(note)),
        }
    }

    fn map_element(&self, elem: UmlElement) -> Node {
        let mut properties = HashMap::new();

        if let Some(stereo) = elem.stereotype {
            properties.insert("stereotype".to_string(), stereo.name);
        }

        Node {
            id: elem.id.0,
            label: elem.display_name.or(elem.alias),
            node_type: Self::map_node_type(elem.kind),
            properties,
        }
    }

    fn map_node_type(kind: UmlElementKind) -> NodeType {
        match kind {
            UmlElementKind::Class | UmlElementKind::AbstractClass => NodeType::Class,
            UmlElementKind::Interface => NodeType::Interface,
            UmlElementKind::Actor => NodeType::Actor,
            UmlElementKind::Database => NodeType::Database,
            UmlElementKind::Enum | UmlElementKind::Component => NodeType::Default,
        }
    }

    fn map_relation(&self, rel: UmlRelation) -> Edge {
        Edge {
            from: rel.from.0,
            to: rel.to.0,
            label: rel.label,
            interaction: Self::determine_interaction(&rel.arrow.left, &rel.arrow.right),
            style: EdgeStyle {
                line: Self::map_line_type(rel.arrow.line),
                tail: Self::map_arrow_type(&rel.arrow.left),
                head: Self::map_arrow_type(&rel.arrow.right),
            },
        }
    }

    fn determine_interaction(left: &UmlArrowEnd, right: &UmlArrowEnd) -> InteractionType {
        Self::map_interaction_type(right)
            .or_else(|| Self::map_interaction_type(left))
            .unwrap_or(InteractionType::Association)
    }

    fn map_interaction_type(arrow_end: &UmlArrowEnd) -> Option<InteractionType> {
        match arrow_end {
            UmlArrowEnd::Inheritance => Some(InteractionType::Inheritance),
            UmlArrowEnd::Composition => Some(InteractionType::Composition),
            UmlArrowEnd::Aggregation => Some(InteractionType::Aggregation),
            UmlArrowEnd::Dependency => Some(InteractionType::Dependency),
            UmlArrowEnd::Association => Some(InteractionType::Association),
            UmlArrowEnd::None => None,
        }
    }

    fn map_line_type(line: UmlLineStyle) -> LineType {
        match line {
            UmlLineStyle::Solid => LineType::Solid,
            UmlLineStyle::Dotted => LineType::Dashed,
        }
    }

    fn map_arrow_type(end: &UmlArrowEnd) -> ArrowType {
        match end {
            UmlArrowEnd::None => ArrowType::None,
            UmlArrowEnd::Association | UmlArrowEnd::Dependency => ArrowType::Vee,
            UmlArrowEnd::Inheritance => ArrowType::Triangle,
            UmlArrowEnd::Composition => ArrowType::FilledDiamond,
            UmlArrowEnd::Aggregation => ArrowType::Diamond,
        }
    }

    fn map_package(&mut self, pkg: UmlPackage) -> Cluster {
        let children: Vec<Element> = pkg
            .children
            .into_iter()
            .map(|stmt| self.map_statement(stmt))
            .collect();

        Cluster {
            id: pkg.id.0,
            label: pkg.display_name,
            cluster_type: Self::map_cluster_type(pkg.kind),
            children,
            properties: HashMap::new(),
        }
    }

    fn map_cluster_type(kind: UmlPackageKind) -> ClusterType {
        match kind {
            UmlPackageKind::Package => ClusterType::Package,
            UmlPackageKind::Namespace => ClusterType::Namespace,
            UmlPackageKind::Folder => ClusterType::Folder,
            UmlPackageKind::Rectangle => ClusterType::Rectangle,
            UmlPackageKind::Frame => ClusterType::Frame,
            UmlPackageKind::Node => ClusterType::Subgraph,
        }
    }

    fn map_note(&mut self, note: UmlNote) -> Note {
        self.note_counter += 1;
        let id = format!("note_{}", self.note_counter);

        Note {
            id,
            text: note.text,
            position: Self::map_note_position(note.position),
            target_node_id: note.target.map(|id| id.0),
        }
    }

    fn map_note_position(pos: UmlNotePosition) -> NotePosition {
        match pos {
            UmlNotePosition::Left => NotePosition::Left,
            UmlNotePosition::Right => NotePosition::Right,
            UmlNotePosition::Top => NotePosition::Top,
            UmlNotePosition::Bottom => NotePosition::Bottom,
            UmlNotePosition::Over => NotePosition::Over,
            UmlNotePosition::Floating => NotePosition::Floating,
        }
    }
}

// ---------------------------------------------------------
// Unit Tests (TDD phase)
// ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::infra::ast::plant_uml_ast::{UmlArrow, UmlId, UmlStereotype};

    use super::*;
    // Assuming imported items from AST and Diagram
    // use crate::plant_uml_ast::*;
    // use crate::diagram::*;

    #[test]
    fn test_map_empty_ast() {
        let ast = PlantUmlAst {
            header: None,
            statements: vec![],
        };

        let mut mapper = AstToDiagramMapper::new();
        let diagram = mapper.map(ast);

        assert_eq!(diagram.kind, DiagramKind::Class);
        assert!(diagram.elements.is_empty());
    }

    #[test]
    fn test_map_element_to_node() {
        let ast = PlantUmlAst {
            header: None,
            statements: vec![UmlStatement::Element(UmlElement {
                kind: UmlElementKind::Database,
                id: UmlId("db1".to_string()),
                display_name: Some("Main DB".to_string()),
                alias: None,
                stereotype: Some(UmlStereotype {
                    name: "mysql".to_string(),
                }),
                members: vec![],
                modifiers: vec![],
            })],
        };

        let mut mapper = AstToDiagramMapper::new();
        let diagram = mapper.map(ast);

        assert_eq!(diagram.elements.len(), 1);

        if let Element::Node(node) = &diagram.elements[0] {
            assert_eq!(node.id, "db1");
            assert_eq!(node.label, Some("Main DB".to_string()));
            assert_eq!(node.node_type, NodeType::Database);
            assert_eq!(node.properties.get("stereotype").unwrap(), "mysql");
        } else {
            panic!("Expected Node");
        }
    }

    #[test]
    fn test_map_relation_to_edge() {
        let ast = PlantUmlAst {
            header: None,
            statements: vec![UmlStatement::Relation(UmlRelation {
                from: UmlId("User".to_string()),
                to: UmlId("Profile".to_string()),
                label: Some("has".to_string()),
                arrow: UmlArrow {
                    line: UmlLineStyle::Solid,
                    left: UmlArrowEnd::Composition, // *
                    right: UmlArrowEnd::None,       // --
                },
            })],
        };

        let mut mapper = AstToDiagramMapper::new();
        let diagram = mapper.map(ast);

        if let Element::Edge(edge) = &diagram.elements[0] {
            assert_eq!(edge.from, "User");
            assert_eq!(edge.to, "Profile");
            assert_eq!(edge.label, Some("has".to_string()));
            assert_eq!(edge.interaction, InteractionType::Composition);
            assert_eq!(edge.style.line, LineType::Solid);
            assert_eq!(edge.style.tail, ArrowType::FilledDiamond); // Left maps to tail
            assert_eq!(edge.style.head, ArrowType::None); // Right maps to head
        } else {
            panic!("Expected Edge");
        }
    }

    #[test]
    fn test_map_package_to_cluster_with_children() {
        let ast = PlantUmlAst {
            header: None,
            statements: vec![UmlStatement::Package(UmlPackage {
                kind: UmlPackageKind::Folder,
                id: UmlId("auth_folder".to_string()),
                display_name: Some("Auth".to_string()),
                children: vec![UmlStatement::Element(UmlElement {
                    kind: UmlElementKind::Class,
                    id: UmlId("User".to_string()),
                    display_name: None,
                    alias: None,
                    stereotype: None,
                    members: vec![],
                    modifiers: vec![],
                })],
            })],
        };

        let mut mapper = AstToDiagramMapper::new();
        let diagram = mapper.map(ast);

        if let Element::Cluster(cluster) = &diagram.elements[0] {
            assert_eq!(cluster.id, "auth_folder");
            assert_eq!(cluster.label, Some("Auth".to_string()));
            assert_eq!(cluster.cluster_type, ClusterType::Folder);
            assert_eq!(cluster.children.len(), 1);

            if let Element::Node(child_node) = &cluster.children[0] {
                assert_eq!(child_node.id, "User");
                assert_eq!(child_node.node_type, NodeType::Class);
            } else {
                panic!("Expected child to be a Node");
            }
        } else {
            panic!("Expected Cluster");
        }
    }

    #[test]
    fn test_map_notes_with_deterministic_ids() {
        let ast = PlantUmlAst {
            header: None,
            statements: vec![
                UmlStatement::Note(UmlNote {
                    text: "First note".to_string(),
                    position: UmlNotePosition::Right,
                    target: Some(UmlId("User".to_string())),
                }),
                UmlStatement::Note(UmlNote {
                    text: "Second note".to_string(),
                    position: UmlNotePosition::Floating,
                    target: None,
                }),
            ],
        };

        let mut mapper = AstToDiagramMapper::new();
        let diagram = mapper.map(ast);

        if let Element::Note(note1) = &diagram.elements[0] {
            assert_eq!(note1.id, "note_1"); // Validates stateful counter
            assert_eq!(note1.text, "First note");
            assert_eq!(note1.position, NotePosition::Right);
            assert_eq!(note1.target_node_id, Some("User".to_string()));
        }

        if let Element::Note(note2) = &diagram.elements[1] {
            assert_eq!(note2.id, "note_2"); // Validates increment
            assert_eq!(note2.position, NotePosition::Floating);
            assert_eq!(note2.target_node_id, None);
        }
    }

    #[test]
    fn test_dynamic_diagram_kind_inference() {
        // Build an AST with an Interface nested inside a Namespace.
        // The engine should recurse into the package and deduce it's a Class Diagram.
        let ast = PlantUmlAst {
            header: None,
            statements: vec![UmlStatement::Package(UmlPackage {
                kind: UmlPackageKind::Namespace,
                id: UmlId("Core".to_string()),
                display_name: None,
                children: vec![UmlStatement::Element(UmlElement {
                    kind: UmlElementKind::Interface, // The "tell" that this is a Class diagram
                    id: UmlId("Repository".to_string()),
                    display_name: None,
                    alias: None,
                    stereotype: None,
                    members: vec![],
                    modifiers: vec![],
                })],
            })],
        };

        let mut mapper = AstToDiagramMapper::new();
        let diagram = mapper.map(ast);

        assert_eq!(diagram.kind, DiagramKind::Class);
    }
}
