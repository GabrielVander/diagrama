use std::collections::HashMap;

use crate::entities::{id::Id, style::StyleRef, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: Id,
    pub kind: NodeKind,
    pub label: Option<String>,
    pub data: HashMap<String, Value>,
    pub style: StyleRef,
    pub parent: Option<Id>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Entity,
    Interface,
    Actor,
    Component,
    Database,
    Group,
    Annotation,
    Custom(String),
}
