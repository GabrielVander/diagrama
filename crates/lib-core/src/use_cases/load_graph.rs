use crate::{
    adapters::graph_gateway::{GraphGateway, GraphGatewayError},
    entities::graph::Graph,
};

pub struct LoadGraph<'a, T: GraphGateway + Sync> {
    pub diagram_parser: &'a T,
}

impl<'a, T: GraphGateway + Sync> LoadGraph<'a, T> {
    pub async fn execute(&self, source: &str) -> Result<Graph, String> {
        self.diagram_parser
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
    use async_trait::async_trait;

    use crate::{
        entities::graph::Graph,
        use_cases::load_graph::{GraphGateway, GraphGatewayError, LoadGraph},
    };

    #[test]
    fn should_delegate_parsing_to_parser() {
        smol::block_on(async {
            let source: &str = "Some source";
            let diagram: Graph = Graph::default();
            let parser: FakeGraphReader = FakeGraphReader::returning(Ok(diagram.clone()));

            let use_case: LoadGraph<FakeGraphReader> = LoadGraph {
                diagram_parser: &parser,
            };

            let result: Result<Graph, String> = use_case.execute(source).await;

            assert_eq!(Ok(diagram.clone()), result);
        });
    }

    #[test]
    fn should_parse_parser_error() {
        smol::block_on(async {
            let source: &str = "Some other source";
            let parser_error: GraphGatewayError = GraphGatewayError::Parse {
                source: "fake".to_owned(),
                message: "dummy error".to_owned(),
                line: 3,
                column: 33,
            };

            let parser: FakeGraphReader = FakeGraphReader::returning(Err(parser_error.clone()));

            let use_case: LoadGraph<FakeGraphReader> = LoadGraph {
                diagram_parser: &parser,
            };

            let result: Result<Graph, String> = use_case.execute(source).await;

            assert_eq!(
                Err("[fake:3:33] Parse Error: dummy error".to_owned()),
                result
            );
        });
    }

    struct FakeGraphReader {
        parse_result: Result<Graph, GraphGatewayError>,
    }

    impl FakeGraphReader {
        fn returning(parse_result: Result<Graph, GraphGatewayError>) -> Self {
            Self { parse_result }
        }
    }

    #[async_trait]
    impl GraphGateway for FakeGraphReader {
        async fn read_graph_from_raw_input(
            &self,
            _source: &str,
        ) -> Result<Graph, GraphGatewayError> {
            self.parse_result.clone()
        }
    }
}
