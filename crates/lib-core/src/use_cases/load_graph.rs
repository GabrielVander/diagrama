use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    adapters::graph_gateway::{GraphGateway, GraphGatewayError},
    entities::graph::Graph,
};

#[async_trait]
pub trait LoadGraphUseCase {
    async fn execute(&self, source: &str) -> Result<Graph, String>;
}

pub struct LoadGraph<T: GraphGateway> {
    graph_gateway: Arc<T>,
}

impl<T: GraphGateway> LoadGraph<T> {
    pub fn new(graph_gateway: Arc<T>) -> Self {
        Self { graph_gateway }
    }
}

#[async_trait]
impl<T: GraphGateway + Sync + Send + 'static> LoadGraphUseCase for LoadGraph<T> {
    async fn execute(&self, source: &str) -> Result<Graph, String> {
        self.graph_gateway
            .read_graph_from_raw_input(source)
            .await
            .map_err(String::from)
    }
}

impl From<GraphGatewayError> for String {
    fn from(value: GraphGatewayError) -> Self {
        match value {
            GraphGatewayError::Parse {
                source,
                message,
                line,
                column,
            } => format!("[{}:{}:{}] Parse Error: {}", source, line, column, message),
            GraphGatewayError::Semantic { source, message } => {
                format!("[{}] Semantic Error: {}", source, message)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::{
        entities::graph::Graph,
        use_cases::load_graph::{GraphGateway, GraphGatewayError, LoadGraph, LoadGraphUseCase},
    };

    macro_rules! async_test {
        ($body:expr) => {
            smol::block_on(async { $body })
        };
    }

    #[test]
    fn should_delegate_reading_to_gateway() {
        async_test!({
            let source: &str = "Some source";
            let diagram: Graph = Graph::default();
            let gateway: Arc<FakeGraphGateway> =
                Arc::new(FakeGraphGateway::returning(Ok(diagram.clone())));

            let use_case: LoadGraph<FakeGraphGateway> = LoadGraph::new(gateway.clone());

            let result: Result<Graph, String> = use_case.execute(source).await;

            assert_eq!(Ok(diagram.clone()), result);
            assert_eq!(Some(source.to_owned()), gateway.received_input())
        });
    }

    #[test]
    fn should_parse_gateway_error() {
        async_test!({
            let source: &str = "Some other source";
            let gateway: Arc<FakeGraphGateway> =
                Arc::new(FakeGraphGateway::returning(Err(GraphGatewayError::Parse {
                    source: "fake".to_owned(),
                    message: "dummy error".to_owned(),
                    line: 3,
                    column: 33,
                }
                .clone())));

            let use_case: LoadGraph<FakeGraphGateway> = LoadGraph::new(gateway.clone());

            let result: Result<Graph, String> = use_case.execute(source).await;

            assert_eq!(
                Err("[fake:3:33] Parse Error: dummy error".to_owned()),
                result
            );
            assert_eq!(Some(source.to_owned()), gateway.received_input())
        });
    }

    struct FakeGraphGateway {
        result: Result<Graph, GraphGatewayError>,
        received_input: Mutex<Option<String>>,
    }

    impl FakeGraphGateway {
        fn returning(result: Result<Graph, GraphGatewayError>) -> Self {
            Self {
                result,
                received_input: Mutex::new(None),
            }
        }

        fn received_input(&self) -> Option<String> {
            self.received_input
                .lock()
                .unwrap()
                .as_deref()
                .map(|i| i.to_owned())
        }
    }

    #[async_trait]
    impl GraphGateway for FakeGraphGateway {
        async fn read_graph_from_raw_input(
            &self,
            source: &str,
        ) -> Result<Graph, GraphGatewayError> {
            *self.received_input.lock().unwrap() = Some(source.to_owned());
            self.result.clone()
        }
    }
}
