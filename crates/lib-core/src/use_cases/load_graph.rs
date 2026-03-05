use async_trait::async_trait;

use crate::entities::graph::Graph;

#[async_trait]
pub trait GraphReader {
    async fn read(&self, source: &str) -> Result<Graph, GraphReaderError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphReaderError {
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

pub struct LoadGraph<'a, T: GraphReader> {
    pub diagram_parser: &'a T,
}

impl<'a, T: GraphReader> LoadGraph<'a, T> {
    pub async fn execute(&self, source: &str) -> Result<Graph, GraphLoadingError> {
        self.diagram_parser
            .read(source)
            .await
            .map_err(GraphLoadingError::from)
    }
}

impl From<GraphReaderError> for GraphLoadingError {
    fn from(value: GraphReaderError) -> Self {
        match value {
            GraphReaderError::Parse {
                source,
                message,
                line,
                column,
            } => Self(format!(
                "[{}:{}:{}] Parse Error: {}",
                source, line, column, message
            )),
            GraphReaderError::Semantic { source, message } => {
                Self(format!("[{}] Semantic Error: {}", source, message))
            }
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct GraphLoadingError(String);

#[cfg(test)]
mod test {
    use async_lock::{Mutex, MutexGuard};
    use async_trait::async_trait;

    use crate::{
        entities::graph::Graph,
        use_cases::load_graph::{GraphLoadingError, GraphReader, GraphReaderError, LoadGraph},
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

            let result: Result<Graph, GraphLoadingError> = use_case.execute(source).await;

            assert_eq!(Ok(diagram.clone()), result);
        });
    }

    #[test]
    fn should_parse_parser_error() {
        smol::block_on(async {
            let source: &str = "Some other source";
            let parser_error: GraphReaderError = GraphReaderError::Parse {
                source: "fake".to_owned(),
                message: "dummy error".to_owned(),
                line: 3,
                column: 33,
            };

            let parser: FakeGraphReader = FakeGraphReader::returning(Err(parser_error.clone()));

            let use_case: LoadGraph<FakeGraphReader> = LoadGraph {
                diagram_parser: &parser,
            };

            let result: Result<Graph, GraphLoadingError> = use_case.execute(source).await;

            assert_eq!(
                Err(GraphLoadingError(
                    "[fake:3:33] Parse Error: dummy error".to_owned()
                )),
                result
            );
        });
    }

    struct FakeGraphReader {
        last_parse_input: Mutex<Option<String>>,

        parse_result: Result<Graph, GraphReaderError>,
    }

    impl FakeGraphReader {
        fn returning(parse_result: Result<Graph, GraphReaderError>) -> Self {
            Self {
                last_parse_input: Mutex::new(None),
                parse_result,
            }
        }
    }

    #[async_trait]
    impl GraphReader for FakeGraphReader {
        async fn read(&self, source: &str) -> Result<Graph, GraphReaderError> {
            let mut guard: MutexGuard<Option<String>> = self.last_parse_input.lock().await;
            *guard = Some(source.to_string());

            self.parse_result.clone()
        }
    }
}
