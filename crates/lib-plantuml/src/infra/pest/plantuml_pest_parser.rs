use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "infra/pest/plantuml.pest"]
pub struct PlantumlPestParser;
