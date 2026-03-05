use std::collections::HashMap;

use crate::entities::{id::Id, style::StyleRef, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: Id,
    pub from: Id,
    pub to: Id,
    pub directed: bool,
    pub kind: EdgeKind,
    pub label: Option<String>,
    pub data: HashMap<String, Value>,
    pub style: StyleRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeKind {
    Association,
    Dependency,
    Inheritance,
    Aggregation,
    Composition,
    Flow,
    Undirected,
    Custom(String),
}
