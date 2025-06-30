use crate::pg::pg::PgConnect;
use crate::sui;
use anyhow::anyhow;
use regex::Regex;
use serde::__private::de::IdentifierDeserializer;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use tracing::{error, info};
use uuid::Uuid;

type GlobalError = Box<dyn Error>;
#[derive(Debug)]
pub struct ChatMessage {
    id: Option<String>,
    pub command: MessageCommands,
    user_id: i64,
    status: MessageStatus,
}

#[derive(Debug)]
pub enum MessageStatus {
    AWAITING,
    IN_PROCESS,
    COMPLETED,
}

#[derive(Debug)]
pub enum MessageCommands {
    STORE_CHAT_MESSAGE(String),
    Unknown(String),
}
impl Display for MessageCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageCommands::STORE_CHAT_MESSAGE(s) => "!STORE".to_string(),
            MessageCommands::Unknown(s) => "UNKNOWN".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageStatus::AWAITING => "AWAITING".to_string(),
            MessageStatus::COMPLETED => "COMPLETED".to_string(),
            MessageStatus::IN_PROCESS => "IN_PROCESS".to_string(),
        };
        write!(f, "{}", str)
    }
}
impl FromStr for MessageCommands {
    type Err = GlobalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reg = Regex::new(r"^(![A-Z]+)\s(.*)")?;
        let captures = reg.captures(s).ok_or("Failed to capture")?;
        // todo clean up unwraps
        let command = captures.get(1).ok_or("Failed to capture command")?;
        let text = captures
            .get(2)
            .ok_or("Failed to capture text")?
            .as_str()
            .to_string();
        let parsed_command = match command.as_str() {
            "!PROMPT" => MessageCommands::STORE_CHAT_MESSAGE(text),
            _ => MessageCommands::Unknown(text),
        };
        Ok(parsed_command)
    }
}
impl ChatMessage {
    pub fn new_from_db(id: String, command: MessageCommands, user_id: i64) -> Self {
        Self {
            id: Some(id),
            command,
            user_id,
            status: MessageStatus::AWAITING,
        }
    }
    pub fn new_from_raw(full_message: String, user_id: i64) -> anyhow::Result<Self> {
        match Self::parse(full_message.clone()) {
            Ok(command) => Ok(Self {
                id: None,
                command,
                user_id,
                status: MessageStatus::AWAITING,
            }),
            Err(e) => {
                error!("Error parsing message {}", full_message);
                Err(anyhow!("Error parsing in new_from_Raw"))
            }
        }
    }

    fn parse(full_message: String) -> Result<MessageCommands, GlobalError> {
        MessageCommands::from_str(&full_message)
    }

    pub async fn verify_and_send(&self) -> Result<(), GlobalError> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        match &self.command {
            MessageCommands::STORE_CHAT_MESSAGE(text) => {
                let query = "INSERT INTO chat_messages ( user_id, text, command, status ) VALUES ($1, $2, $3, $4)";
                client
                    .query(
                        query,
                        &[
                            &self.user_id,
                            text,
                            &self.command.to_string(),
                            &MessageStatus::AWAITING.to_string(),
                        ],
                    )
                    .await?;
            }
            MessageCommands::Unknown(text) => {
                info!("Skipping message {}", text);
            }
        };

        Ok(())
    }

    pub async fn update_status(&self, status: MessageStatus) -> anyhow::Result<()> {
        let pool = PgConnect::create_pool_from_env().unwrap();
        let client = pool.get().await?;
        let query = "UPDATE chat_messages SET status = $1 WHERE id = $2";
        let id = Uuid::from_str(self.id.as_ref().unwrap())?;
        client.query(query, &[&status.to_string(), &id]).await?;
        Ok(())
    }
}
