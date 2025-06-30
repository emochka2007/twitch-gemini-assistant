use crate::pg::pg::PgConnect;
use crate::terminal::test_send_to_terminal;
use crate::twitch::chat_message::{ChatMessage, MessageCommands, MessageStatus};
use std::str::FromStr;
use tokio::time::{Duration, interval};
use tracing::{error, info};
use uuid::Uuid;

pub struct EventPoller {}

impl EventPoller {
    pub async fn poll_message() -> anyhow::Result<ChatMessage> {
        let pool = PgConnect::create_pool_from_env().unwrap();
        let client = pool.get().await?;
        let query =
            "SELECT * FROM chat_messages WHERE status='AWAITING' ORDER BY created_at LIMIT 1";
        let message = client.query_one(query, &[]).await?;
        let id: Uuid = message.try_get("id")?;
        let command: String = message.try_get("command")?;
        let text: String = message.try_get("text")?;
        let concat_command = format!("{} {}", command, text);
        Ok(ChatMessage::new_from_db(
            id.to_string(),
            MessageCommands::from_str(&concat_command).unwrap(),
            message.try_get("user_id")?,
        ))
    }

    pub async fn init() -> anyhow::Result<()> {
        let mut ticker = interval(Duration::from_secs(70));

        loop {
            // wait until the next tick
            ticker.tick().await;

            match EventPoller::poll_message().await {
                Ok(msg) => {
                    info!("Got message: {:?}", msg);
                    match msg.command {
                        MessageCommands::Unknown(ref text) => {
                            test_send_to_terminal(text)?;
                            msg.update_status(MessageStatus::COMPLETED).await?;
                        }
                        _ => {
                            error!("Skipping message {:?}", msg);
                        }
                    }
                }
                Err(err) => {
                    error!("[ERROR polling message]: {:?}", err);
                }
            }
        }
    }
}
