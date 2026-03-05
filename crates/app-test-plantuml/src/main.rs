use std::io::{self, Read};

use lib_core::use_cases::load_graph::LoadGraph;
use lib_plantuml::adapters::graph_parser_plantuml_impl::GraphParserPlantumlImpl;

fn main() {
    let adapter: GraphParserPlantumlImpl = GraphParserPlantumlImpl::new();
    let use_case: LoadGraph<GraphParserPlantumlImpl> = LoadGraph {
        diagram_parser: &adapter,
    };

    let mut input: String = String::new();

    io::stdin().read_to_string(&mut input).unwrap();
    println!("{:?}", smol::block_on(use_case.execute(&input)))
}
