use lib_core::entities::{
    edge::{Edge, EdgeKind},
    graph::Graph,
    group::Group,
    id::Id,
    node::{Node, NodeKind},
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::infra::models::ast_node::AstNode;

pub struct GraphBuilder {
    graph: Graph,
    alias_map: HashMap<String, String>, // Maps PlantUML aliases to actual Node IDs
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: Graph {
                id: Uuid::new_v4().to_string(),
                ..Default::default()
            },
            alias_map: HashMap::new(),
        }
    }

    pub fn build(mut self, ast: Vec<AstNode>) -> Graph {
        ast.iter().for_each(|node: &AstNode| {
            self.process_ast_node(node, None);
        });
        self.graph
    }

    fn process_ast_node(&mut self, node: &AstNode, parent_id: Option<String>) {
        match node {
            AstNode::Definition {
                keyword,
                name,
                alias,
            } => {
                let id: String = alias.clone().unwrap_or_else(|| name.clone());

                if let Some(a) = alias {
                    self.alias_map.insert(a.clone(), id.clone());
                }

                let kind: NodeKind = match keyword.as_str() {
                    "class" => NodeKind::Entity,
                    "interface" => NodeKind::Interface,
                    "actor" => NodeKind::Actor,
                    "component" => NodeKind::Component,
                    "database" => NodeKind::Database,
                    _ => NodeKind::Custom(keyword.clone()),
                };

                self.graph.nodes.insert(
                    id.clone(),
                    Node {
                        id: id.clone(),
                        kind,
                        label: Some(name.clone()),
                        data: HashMap::new(),
                        style: None,
                        parent: parent_id,
                    },
                );
            }
            AstNode::Relation {
                left,
                right,
                arrow,
                label,
            } => {
                let left_id: String = self.resolve_id(&left);
                let right_id: String = self.resolve_id(&right);

                // Ensure implicit nodes exist
                self.ensure_node_exists(&left_id);
                self.ensure_node_exists(&right_id);

                let (kind, directed): (EdgeKind, bool) = self.map_arrow(&arrow);

                let edge_id: String = Uuid::new_v4().to_string();
                self.graph.edges.insert(
                    edge_id.clone(),
                    Edge {
                        id: edge_id,
                        from: left_id,
                        to: right_id,
                        directed,
                        kind,
                        label: label.clone(),
                        data: HashMap::new(),
                        style: None,
                    },
                );
            }
            AstNode::Package { name, children } => {
                let group_id: String = Uuid::new_v4().to_string();
                let mut child_ids: Vec<Id> = Vec::new();

                children.iter().for_each(|child: &AstNode| {
                    // Quick peek to grab IDs for the group's child list
                    if let AstNode::Definition {
                        alias,
                        name: child_name,
                        ..
                    } = &child
                    {
                        child_ids.push(alias.clone().unwrap_or_else(|| child_name.clone()));
                    }
                    self.process_ast_node(child, Some(group_id.clone()));
                });

                self.graph.groups.insert(
                    group_id.clone(),
                    Group {
                        id: group_id,
                        label: Some(name.clone()),
                        children: child_ids,
                        parent: parent_id,
                    },
                );
            }
        }
    }

    fn resolve_id(&self, identifier: &str) -> String {
        self.alias_map
            .get(identifier)
            .cloned()
            .unwrap_or_else(|| identifier.to_string())
    }

    fn ensure_node_exists(&mut self, id: &str) {
        if !self.graph.nodes.contains_key(id) {
            self.graph.nodes.insert(
                id.to_string(),
                Node {
                    id: id.to_string(),
                    kind: NodeKind::Entity, // Default kind for implicit nodes
                    label: Some(id.to_string()),
                    data: HashMap::new(),
                    style: None,
                    parent: None,
                },
            );
        }
    }

    fn map_arrow(&self, arrow: &str) -> (EdgeKind, bool) {
        match arrow {
            "-->" | "<--" => (EdgeKind::Association, true),
            "--|>" | "<|--" => (EdgeKind::Inheritance, true),
            "--*" | "*--" => (EdgeKind::Composition, true),
            "--o" | "o--" => (EdgeKind::Aggregation, true),
            "--" => (EdgeKind::Undirected, false),
            _ => (EdgeKind::Custom(arrow.to_string()), true),
        }
    }
}
