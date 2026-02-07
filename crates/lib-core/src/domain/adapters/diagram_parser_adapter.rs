use crate::domain::entities::diagram::Diagram;

pub trait DiagramParserAdapter {
    fn parse(&self, source: &str) -> Result<Diagram, String>;
}
