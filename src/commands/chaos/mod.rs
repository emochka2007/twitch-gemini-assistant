use crate::command_framework::command::Command;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tracing::info;

#[derive(Debug)]
pub struct ChaosCommand {
    pool: Pool,
}

#[async_trait]
impl Command for ChaosCommand {
    fn name(&self) -> String {
        "chaos".to_owned()
    }

    async fn action(&self, _content: Option<String>) -> anyhow::Result<()> {
        info!("Chaos action");
        Ok(())
    }
}

impl ChaosCommand {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}
