use async_trait::async_trait;

use crate::domain::entities::graph::Graph;

#[async_trait]
pub trait GraphParser {
    async fn parse(&self, source: &str) -> Result<Graph, FrontendError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrontendError {
    Parse {
        source: String,
        message: String,
        line: usize,
        column: usize,
    },
    Semantic {
        source: String,
        message: String,
    },
}
