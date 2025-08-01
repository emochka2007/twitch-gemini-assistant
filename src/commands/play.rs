use crate::command_framework::command::Command;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tracing::info;

#[derive(Debug)]
pub struct PlayCommand {
    pool: Pool,
}

#[async_trait]
impl Command for PlayCommand {
    fn name(&self) -> String {
        "play".to_owned()
    }

    async fn action(&self, content: Option<String>) -> anyhow::Result<()> {
        info!("Play action, content : {:?}", content);
        Ok(())
    }
}

impl PlayCommand {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}
