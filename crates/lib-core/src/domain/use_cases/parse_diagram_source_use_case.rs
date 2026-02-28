use crate::domain::{
    adapters::graph_parser::{FrontendError, GraphParser},
    entities::graph::Graph,
};

pub struct ParseDiagramSourceUseCase<'a, T: GraphParser> {
    diagram_parser: &'a T,
}

impl<'a, T: GraphParser> ParseDiagramSourceUseCase<'a, T> {
    async fn execute(&self, source: &str) -> Result<Graph, ParseDiagramSourceError> {
        self.diagram_parser
            .parse(source)
            .await
            .map_err(|e: FrontendError| ParseDiagramSourceError::ParserError {
                context: format!("{:?}", e),
            })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseDiagramSourceError {
    ParserError { context: String },
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use async_lock::{Mutex, MutexGuard};
    use async_trait::async_trait;
    use pretty_assertions::assert_eq;

    use crate::domain::{
        adapters::graph_parser::{FrontendError, GraphParser},
        entities::graph::Graph,
        use_cases::parse_diagram_source_use_case::{
            ParseDiagramSourceError, ParseDiagramSourceUseCase,
        },
    };

    #[test]
    fn should_delegate_parsing_to_parser() {
        smol::block_on(async {
            let source: &str = "Some source";
            let diagram: Graph = Graph::default();
            let parser: FakeGraphParser = FakeGraphParser::returning(Ok(diagram.clone()));

            let use_case: ParseDiagramSourceUseCase<FakeGraphParser> = ParseDiagramSourceUseCase {
                diagram_parser: &parser,
            };

            let result: Result<Graph, ParseDiagramSourceError> = use_case.execute(source).await;

            parser.assert_parse_called_with(source).await;
            assert_eq!(Ok(diagram.clone()), result);
        });
    }

    #[test]
    fn should_parse_parser_error() {
        smol::block_on(async {
            let source: &str = "Some other source";
            let parser_error: FrontendError = FrontendError::Parse {
                source: "fake".to_owned(),
                message: "dummy error".to_owned(),
                line: 3,
                column: 33,
            };

            let parser: FakeGraphParser = FakeGraphParser::returning(Err(parser_error.clone()));

            let use_case: ParseDiagramSourceUseCase<FakeGraphParser> = ParseDiagramSourceUseCase {
                diagram_parser: &parser,
            };

            let result: Result<Graph, ParseDiagramSourceError> = use_case.execute(source).await;

            parser.assert_parse_called_with(source).await;
            assert_eq!(
                Err(ParseDiagramSourceError::ParserError {
                    context: format!("{:?}", parser_error)
                }),
                result
            );
        });
    }

    struct FakeGraphParser {
        last_parse_input: Mutex<Option<String>>,

        parse_result: Result<Graph, FrontendError>,
    }

    impl FakeGraphParser {
        fn returning(parse_result: Result<Graph, FrontendError>) -> Self {
            Self {
                last_parse_input: Mutex::new(None),
                parse_result,
            }
        }

        async fn assert_parse_called_with(&self, expected: &str) {
            let guard: MutexGuard<Option<String>> = self.last_parse_input.lock().await;
            let actual: Option<&str> = guard.as_deref();

            assert_eq!(Some(expected), actual)
        }
    }

    #[async_trait]
    impl GraphParser for FakeGraphParser {
        async fn parse(&self, source: &str) -> Result<Graph, FrontendError> {
            let mut guard: MutexGuard<Option<String>> = self.last_parse_input.lock().await;
            *guard = Some(source.to_string());

            self.parse_result.clone()
        }
    }
}
