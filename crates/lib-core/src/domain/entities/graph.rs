use std::collections::HashMap;

use crate::domain::entities::{edge::Edge, group::Group, id::Id, node::Node, style::Style};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Graph {
    pub id: Id,
    pub metadata: Metadata,
    pub nodes: HashMap<Id, Node>,
    pub edges: HashMap<Id, Edge>,
    pub groups: HashMap<Id, Group>,
    pub styles: HashMap<Id, Style>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: HashMap<String, String>,
}
