use async_trait::async_trait;

use crate::domain::entities::diagram::Diagram;

#[async_trait]
pub trait DiagramParserAdapter {
    async fn parse(&self, source: &str) -> Result<Diagram, String>;
}
