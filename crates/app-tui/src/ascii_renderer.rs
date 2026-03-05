use lib_core::entities::graph::Graph;
use lib_core::entities::group::Group;
use lib_core::entities::{edge::Edge, node::Node};

pub fn render_graph(graph: &Graph) -> String {
    let mut output = String::new();

    if let Some(ref title) = graph.metadata.title {
        output.push_str(&format!("=== {} ===\n\n", title));
    }

    if !graph.nodes.is_empty() {
        output.push_str("--- Nodes ---\n");
        for node in graph.nodes.values() {
            output.push_str(&render_node(node));
        }
        output.push('\n');
    }

    if !graph.edges.is_empty() {
        output.push_str("--- Edges ---\n");
        for edge in graph.edges.values() {
            output.push_str(&render_edge(edge, graph));
        }
        output.push('\n');
    }

    if !graph.groups.is_empty() {
        output.push_str("--- Groups ---\n");
        for group in graph.groups.values() {
            output.push_str(&render_group(group, graph));
        }
    }

    if output.is_empty() {
        output.push_str("(empty graph)");
    }

    output
}

fn render_node(node: &Node) -> String {
    let label = node.label.as_deref().unwrap_or(&node.id);
    let kind = format!("{:?}", node.kind);
    format!("[{}] {} ({})\n", node.id, label, kind)
}

fn render_edge(edge: &Edge, graph: &Graph) -> String {
    let from_label = graph
        .nodes
        .get(&edge.from)
        .and_then(|n| n.label.clone())
        .unwrap_or_else(|| edge.from.clone());

    let to_label = graph
        .nodes
        .get(&edge.to)
        .and_then(|n| n.label.clone())
        .unwrap_or_else(|| edge.to.clone());

    let arrow = if edge.directed { "-->" } else { "--" };

    let label = edge
        .label
        .as_ref()
        .map(|l| format!(": {}", l))
        .unwrap_or_default();

    format!("{} {} {}{}\n", from_label, arrow, to_label, label)
}

fn render_group(group: &Group, graph: &Graph) -> String {
    let label = group.label.as_deref().unwrap_or(&group.id);
    let children: Vec<String> = group
        .children
        .iter()
        .filter_map(|id| {
            graph
                .nodes
                .get(id)
                .and_then(|n| n.label.clone())
                .or_else(|| Some(id.clone()))
        })
        .collect();

    format!("{} {{\n  {}\n}}\n", label, children.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_core::entities::edge::{Edge, EdgeKind};
    use lib_core::entities::id::Id;
    use lib_core::entities::node::{Node, NodeKind};
    use std::collections::HashMap;

    #[test]
    fn test_render_empty_graph() {
        let graph = Graph::default();
        let output = render_graph(&graph);
        assert_eq!(output, "(empty graph)");
    }

    #[test]
    fn test_render_single_node() {
        let mut graph = Graph::default();
        graph.nodes.insert(
            Id::from("node1"),
            Node {
                id: Id::from("node1"),
                label: Some("TestNode".to_string()),
                kind: NodeKind::Entity,
                parent: None,
                style: None,
                data: HashMap::new(),
            },
        );

        let output = render_graph(&graph);
        assert!(output.contains("TestNode"));
        assert!(output.contains("Entity"));
    }

    #[test]
    fn test_render_edge() {
        let mut graph = Graph::default();
        let node1 = Node {
            id: Id::from("n1"),
            label: Some("A".to_string()),
            kind: NodeKind::Entity,
            parent: None,
            style: None,
            data: HashMap::new(),
        };
        let node2 = Node {
            id: Id::from("n2"),
            label: Some("B".to_string()),
            kind: NodeKind::Entity,
            parent: None,
            style: None,
            data: HashMap::new(),
        };

        graph.nodes.insert(node1.id.clone(), node1);
        graph.nodes.insert(node2.id.clone(), node2);

        graph.edges.insert(
            Id::from("e1"),
            Edge {
                id: Id::from("e1"),
                from: Id::from("n1"),
                to: Id::from("n2"),
                kind: EdgeKind::Association,
                directed: true,
                label: Some("relates to".to_string()),
                style: None,
                data: HashMap::new(),
            },
        );

        let output = render_graph(&graph);
        assert!(output.contains("A --> B"));
        assert!(output.contains("relates to"));
    }
}
