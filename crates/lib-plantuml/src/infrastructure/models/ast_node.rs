#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Definition {
        keyword: String,
        name: String,
        alias: Option<String>,
    },
    Relation {
        left: String,
        right: String,
        arrow: String,
        label: Option<String>,
    },
    Package {
        name: String,
        children: Vec<AstNode>,
    },
}
