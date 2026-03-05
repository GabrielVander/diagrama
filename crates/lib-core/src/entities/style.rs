use std::collections::HashMap;

use crate::entities::id::Id;

pub type StyleRef = Option<Id>;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub id: Id,
    pub properties: HashMap<String, String>,
}
