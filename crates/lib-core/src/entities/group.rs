use crate::entities::id::Id;

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: Id,
    pub label: Option<String>,
    pub children: Vec<Id>,
    pub parent: Option<Id>,
}
