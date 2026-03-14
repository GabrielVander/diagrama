use pest::Parser;
use pest_derive::Parser;

use crate::infrastructure::models::ast_node::AstNode;

#[derive(Parser)]
#[grammar = "infrastructure/plantuml.pest"]
pub struct PlantUmlParser;

pub fn parse_plantuml(input: &str) -> Result<Vec<AstNode>, PlantUmlParseError> {
    let mut ast: Vec<AstNode> = Vec::new();
    let diagram: pest::iterators::Pair<Rule> = PlantUmlParser::parse(Rule::diagram, input)
        .map_err(PlantUmlParseError::from)?
        .next()
        .unwrap();

    diagram
        .into_inner()
        .for_each(|pair: pest::iterators::Pair<Rule>| {
            if let Some(node) = parse_element(pair) {
                ast.push(node);
            }
        });

    Ok(ast)
}

fn parse_element(pair: pest::iterators::Pair<Rule>) -> Option<AstNode> {
    match pair.as_rule() {
        Rule::definition => {
            let mut inner: pest::iterators::Pairs<Rule> = pair.into_inner();
            let keyword: String = inner.next().unwrap().as_str().to_string();
            let name: String = inner.next().unwrap().as_str().trim_matches('"').to_string();
            let alias: Option<String> = inner
                .next()
                .map(|p: pest::iterators::Pair<Rule>| p.as_str().to_string());

            Some(AstNode::Definition {
                keyword,
                name,
                alias,
            })
        }
        Rule::relation => {
            let mut inner: pest::iterators::Pairs<Rule> = pair.into_inner();
            let left: String = inner.next().unwrap().as_str().to_string();
            let arrow: String = inner.next().unwrap().as_str().to_string();
            let right: String = inner.next().unwrap().as_str().to_string();
            let label: Option<String> = inner
                .next()
                .map(|p: pest::iterators::Pair<Rule>| p.as_str().trim_matches('"').to_string());

            Some(AstNode::Relation {
                left,
                right,
                arrow,
                label,
            })
        }
        Rule::package => {
            let mut inner: pest::iterators::Pairs<Rule> = pair.into_inner();
            let name: String = inner.next().unwrap().as_str().trim_matches('"').to_string();
            let mut children: Vec<AstNode> = Vec::new();

            inner.for_each(|child_pair: pest::iterators::Pair<Rule>| {
                if let Some(child) = parse_element(child_pair) {
                    children.push(child);
                }
            });
            Some(AstNode::Package { name, children })
        }
        _ => None,
    }
}

#[derive(Debug)]
pub enum PlantUmlParseError {
    Syntax {
        message: String,
        line: usize,
        column: usize,
    },
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    Internal(String),
}

impl From<pest::error::Error<Rule>> for PlantUmlParseError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        let location: pest::error::LineColLocation = err.line_col.clone();

        let (line, column): (usize, usize) = match location {
            pest::error::LineColLocation::Pos((l, c)) => (l, c),
            pest::error::LineColLocation::Span((l, c), _) => (l, c),
        };

        PlantUmlParseError::Syntax {
            message: err.to_string(),
            line,
            column,
        }
    }
}
