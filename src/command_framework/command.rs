use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait Command: Send + Sync + Debug {
    fn name(&self) -> String;
    async fn action(&self, content: Option<String>) -> anyhow::Result<()>;
}
