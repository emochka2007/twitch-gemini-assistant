use crate::pg::pg::PgConnect;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::Row;
use tracing::info;
use uuid::Uuid;

type GlobalError = Box<dyn Error>;

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    id: Option<Uuid>,
    pub(crate) username: String,
    pub(crate) text: String,
}

impl ChatMessage {
    pub fn new(id: Option<Uuid>, text: String, username: String) -> Self {
        Self { id, text, username }
    }
    pub fn from_raw_message(text: String, username: String) -> Self {
        info!("from_raw {}", text);
        Self::new(None, username, text)
    }

    async fn if_exists(&self) -> anyhow::Result<bool> {
        // схуяли нахуй по тексту
        let query = "SELECT * FROM chat_messages WHERE text = $1";
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let rows = client.query(query, &[&self.text]).await?;
        Ok(rows.len() > 0)
    }

    // async fn is_duplicate_theme(&self) -> anyhow::Result<bool> {
    //     let query = "SELECT * FROM chat_messages WHERE username = $1 and status = $2";
    //     let pool = PgConnect::create_pool_from_env()?;
    //     let client = pool.get().await?;
    //     let rows = client
    //         .query(query, &[&self.username, &self.status.to_string()])
    //         .await?;
    //     Ok(rows.len() > 0)
    // }

    pub async fn insert(&mut self, pool: &deadpool_postgres::Pool) -> Result<(), GlobalError> {
        let client = pool.get().await?;
        info!("Got message: {:?}", self);

        let query = "INSERT INTO chat_messages (username, text) VALUES ($1, $2) RETURNING id";
        let rows = client.query(query, &[&self.username, &self.text]).await?;

        if let Some(row) = rows.get(0) {
            let id: Uuid = row.try_get("id")?;
            self.id = Some(id);
        }
        // TODO: handle error

        Ok(())
    }
    fn row_to_chat_message(row: &Row) -> anyhow::Result<ChatMessage> {
        let id: Uuid = row.try_get("id")?;
        let text: String = row.try_get("text")?;
        let username: String = row.try_get("username")?;
        Ok(ChatMessage::new(Some(id), username, text))
    }
}
