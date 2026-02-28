use std::io::{self, Read};

use lib_core::domain::use_cases::parse_diagram_source_use_case::ParseDiagramSourceUseCase;
use lib_plantuml::adapters::graph_parser_plantuml_impl::GraphParserPlantumlImpl;

fn main() {
    let adapter: GraphParserPlantumlImpl = GraphParserPlantumlImpl::new();
    let use_case: ParseDiagramSourceUseCase<GraphParserPlantumlImpl> = ParseDiagramSourceUseCase {
        diagram_parser: &adapter,
    };

    let mut input: String = String::new();

    io::stdin().read_to_string(&mut input).unwrap();
    println!("{:?}", smol::block_on(use_case.execute(&input)))
}
