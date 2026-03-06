use async_trait::async_trait;

use crate::entities::graph::Graph;

#[async_trait]
pub trait GraphGateway {
    async fn read_graph_from_raw_input(&self, input: &str) -> Result<Graph, GraphGatewayError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphGatewayError {
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
