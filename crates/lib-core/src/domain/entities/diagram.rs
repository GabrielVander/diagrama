use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagram {
    pub title: Option<String>,
    pub kind: DiagramKind,
    pub elements: Vec<Element>,
    pub styles: HashMap<String, String>, // Global styles (generic key-value)
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagramKind {
    Class,
    Sequence,
    Flowchart,
    State,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Node(Node),
    Edge(Edge),
    Cluster(Cluster),
    Note(Note),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub label: Option<String>,
    pub node_type: NodeType,

    // Key-value store for format-specific or extra attributes.
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub interaction: InteractionType,
    pub style: EdgeStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    pub id: String,
    pub label: Option<String>,
    pub cluster_type: ClusterType,
    pub children: Vec<Element>,
    // Generic attributes (color, visual style, etc.)
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub id: String,
    pub text: String,
    pub position: NotePosition,

    // If Some(id), this note connects to a specific Node.
    // If None, it is a "floating" note.
    pub target_node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Default,
    Class,
    Interface,
    Actor,
    Database,
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionType {
    Association, // Plain connection
    Inheritance, // "Is a"
    Composition, // "Has a" (strong)
    Aggregation, // "Has a" (weak)
    Dependency,  // "Uses"
    ControlFlow, // Flowchart arrow
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeStyle {
    pub line: LineType,  // Solid, Dashed, Dotted
    pub head: ArrowType, // None, Open, Filled, Diamond
    pub tail: ArrowType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClusterType {
    Subgraph,
    Package,
    Namespace,
    Rectangle, // Visual grouping only
    Folder,
    Frame,
    Cloud,
    Database, // System boundary
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotePosition {
    Over, // Often used in Sequence diagrams
    Left,
    Right,
    Top,
    Bottom,
    Floating, // Position determined by layout engine
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    Solid,  // ─── (Standard)
    Dashed, // - - (Dependency)
    Dotted, // ... (Weak dependency)
    Bold,   // ═══ (Emphasis)
    Hidden, // Used for layout hacks ( [Hidden] in puml, ~~~ in mermaid)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrowType {
    None, // No head ( --- )

    // Standard Arrows
    Vee,   // -> (Standard association)
    Cross, // x (Lost message / destroy)

    // Inheritance / Realization
    Triangle,       // |> (Inheritance - white inside)
    FilledTriangle, // (Inheritance - black inside, less common)

    // Composition / Aggregation
    Diamond,       // <> (Aggregation - white inside)
    FilledDiamond, // * (Composition - black inside)

    // Connectors
    Circle,       // o  (Socket / Aggregation variant)
    FilledCircle, // * (Solid dot)
    CrowFoot,     // }  (ER Diagrams: One-to-Many)
    HalfOpen,     // \  (Async messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Helper to make map creation less verbose for tests
    fn prop(k: &str, v: &str) -> (String, String) {
        (k.to_string(), v.to_string())
    }

    #[test]
    fn test_construct_complex_nested_structure() {
        // SCENARIO:
        // package "Accounting" {
        //   class Invoice
        //   note right of Invoice: Main financial record
        // }

        let invoice_node = Node {
            id: "invoice_1".to_string(),
            label: Some("Invoice".to_string()),
            node_type: NodeType::Class,
            properties: HashMap::from([prop("visibility", "public"), prop("is_abstract", "false")]),
        };

        let invoice_note = Note {
            id: "note_1".to_string(),
            text: "Main financial record".to_string(),
            position: NotePosition::Right,
            target_node_id: Some("invoice_1".to_string()),
        };

        let accounting_package = Cluster {
            id: "pkg_acc".to_string(),
            label: Some("Accounting".to_string()),
            cluster_type: ClusterType::Package,
            properties: HashMap::from([prop("style", "folder")]),
            children: vec![Element::Node(invoice_node), Element::Note(invoice_note)],
        };

        let diagram = Diagram {
            title: Some("Corporate System".to_string()),
            kind: DiagramKind::Class,
            styles: HashMap::new(),
            elements: vec![Element::Cluster(accounting_package)],
        };

        assert_eq!(diagram.title, Some("Corporate System".to_string()));

        if let Element::Cluster(pkg) = &diagram.elements[0] {
            assert_eq!(pkg.label, Some("Accounting".to_string()));
            assert_eq!(pkg.children.len(), 2);

            match &pkg.children[0] {
                Element::Node(n) => assert_eq!(n.id, "invoice_1"),
                _ => panic!("Expected a Node"),
            }
        } else {
            panic!("Root element should be a Cluster");
        }
    }
}
