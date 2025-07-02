use crate::pg::pg::PgConnect;
use crate::prompt::get_formatted_prompt;
use crate::terminal::test_send_to_terminal;
use crate::twitch::chat_message::{ChatMessage, MessageCommands, MessageStatus};
use anyhow::anyhow;
use std::str::FromStr;
use tokio::time::{Duration, interval};
use tracing::{error, info};
use uuid::Uuid;

pub struct EventPoller {}

impl EventPoller {
    pub async fn is_locked() -> anyhow::Result<bool> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let query = "SELECT * FROM chat_messages WHERE status='IN_PROCESS'";
        let message = client.query(query, &[]).await?;
        Ok(message.len() > 0)
    }

    pub async fn poll_message() -> anyhow::Result<ChatMessage> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let query =
            "SELECT * FROM chat_messages WHERE status='AWAITING' ORDER BY created_at LIMIT 1";
        let message = client.query_one(query, &[]).await?;
        let id: Uuid = message.try_get("id")?;
        let command: String = message.try_get("command")?;
        let text: String = message.try_get("text")?;
        let username: String = message.try_get("username")?;
        Ok(ChatMessage::new(
            Some(String::from(id)),
            text,
            MessageCommands::from_str(&command)?,
            username,
            MessageStatus::Awaiting,
        ))
    }

    pub async fn init() -> anyhow::Result<()> {
        let mut ticker = interval(Duration::from_secs(5));

        loop {
            // wait until the next tick
            ticker.tick().await;

            if EventPoller::is_locked().await? {
                continue;
            }

            match EventPoller::poll_message().await {
                Ok(msg) => {
                    info!("Got message: {:?}", msg);
                    match msg.command {
                        MessageCommands::StoreChatMessage => {
                            let prompt = get_formatted_prompt(&msg.text);
                            msg.update_status(MessageStatus::InProcess).await?;
                            match test_send_to_terminal(&prompt).await {
                                Ok(_) => {
                                    info!("Completed");
                                    msg.update_status(MessageStatus::Completed).await?;
                                }
                                Err(error) => {
                                    error!("Gemini child error {:?}", error);
                                }
                            }
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
