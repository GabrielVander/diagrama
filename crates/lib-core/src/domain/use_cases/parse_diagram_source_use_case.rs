use crate::domain::{
    adapters::diagram_parser_adapter::DiagramParserAdapter, entities::diagram::Diagram,
};

pub struct ParseDiagramSourceUseCase<'a, T: DiagramParserAdapter> {
    diagram_parser: &'a T,
}

impl<'a, T: DiagramParserAdapter> ParseDiagramSourceUseCase<'a, T> {
    async fn execute(&self, source: &str) -> Result<Diagram, ParseDiagramSourceError> {
        self.diagram_parser
            .parse(source)
            .await
            .map_err(|e| ParseDiagramSourceError::ParserError { context: e.clone() })
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
        adapters::diagram_parser_adapter::DiagramParserAdapter,
        entities::diagram::{Diagram, DiagramKind},
        use_cases::parse_diagram_source_use_case::{
            ParseDiagramSourceError, ParseDiagramSourceUseCase,
        },
    };

    // TEST LIST
    //
    // [x] delegates to parser
    // [x] parses parser error

    #[test]
    fn should_delegate_parsing_to_parser() {
        smol::block_on(async {
            let source: &str = "Some source";
            let diagram: Diagram = Diagram {
                title: None,
                kind: DiagramKind::Class,
                elements: Vec::new(),
                styles: HashMap::new(),
            };
            let parser: FakeDiagramParserAdapter =
                FakeDiagramParserAdapter::returning(Ok(diagram.clone()));

            let use_case: ParseDiagramSourceUseCase<FakeDiagramParserAdapter> =
                ParseDiagramSourceUseCase {
                    diagram_parser: &parser,
                };

            let result: Result<Diagram, ParseDiagramSourceError> = use_case.execute(source).await;

            parser.assert_parse_called_with(source).await;
            assert_eq!(Ok(diagram.clone()), result);
        });
    }

    #[test]
    fn should_parse_parser_error() {
        smol::block_on(async {
            let source: &str = "Some other source";
            let parser_error: String = "Some error".to_owned();

            let parser: FakeDiagramParserAdapter =
                FakeDiagramParserAdapter::returning(Err(parser_error.clone()));

            let use_case: ParseDiagramSourceUseCase<FakeDiagramParserAdapter> =
                ParseDiagramSourceUseCase {
                    diagram_parser: &parser,
                };

            let result: Result<Diagram, ParseDiagramSourceError> = use_case.execute(source).await;

            parser.assert_parse_called_with(source).await;
            assert_eq!(
                Err(ParseDiagramSourceError::ParserError {
                    context: parser_error.clone()
                }),
                result
            );
        });
    }

    struct FakeDiagramParserAdapter {
        last_parse_input: Mutex<Option<String>>,

        parse_result: Result<Diagram, String>,
    }

    impl FakeDiagramParserAdapter {
        fn returning(parse_result: Result<Diagram, String>) -> Self {
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
    impl DiagramParserAdapter for FakeDiagramParserAdapter {
        async fn parse(&self, source: &str) -> Result<Diagram, String> {
            let mut guard: MutexGuard<Option<String>> = self.last_parse_input.lock().await;
            *guard = Some(source.to_string());

            self.parse_result.clone()
        }
    }
}
