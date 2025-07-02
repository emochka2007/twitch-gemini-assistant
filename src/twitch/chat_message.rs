use crate::pg::pg::PgConnect;
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
    pub(crate) text: String,
    username: String,
    status: MessageStatus,
}

#[derive(Debug)]
pub enum MessageStatus {
    Awaiting,
    InProcess,
    Completed,
}

#[derive(Debug)]
pub enum MessageCommands {
    StoreChatMessage,
    Unknown,
}
impl Display for MessageCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageCommands::StoreChatMessage => "!STORE".to_string(),
            MessageCommands::Unknown => "UNKNOWN".to_string(),
        };
        write!(f, "{}", str)
    }
}
impl FromStr for MessageCommands {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let command = match s {
            "!STORE" => MessageCommands::StoreChatMessage,
            _ => MessageCommands::Unknown,
        };
        Ok(command)
    }
}

impl Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageStatus::Awaiting => "AWAITING".to_string(),
            MessageStatus::Completed => "COMPLETED".to_string(),
            MessageStatus::InProcess => "IN_PROCESS".to_string(),
        };
        write!(f, "{}", str)
    }
}
impl ChatMessage {
    pub fn new(
        id: Option<String>,
        text: String,
        command: MessageCommands,
        username: String,
        status: MessageStatus,
    ) -> Self {
        Self {
            id,
            command,
            text,
            username,
            status,
        }
    }
    pub fn from_raw_message(full_message: String, username: String) -> anyhow::Result<Self> {
        info!("from_raw {}", full_message);
        match Self::parse(&full_message) {
            Ok((command, text)) => Ok(Self::new(
                None,
                text,
                command,
                username,
                MessageStatus::Awaiting,
            )),
            Err(e) => {
                error!("Error parsing message {:?} - {}", e, full_message);
                Err(anyhow!("Error parsing in new_from_Raw"))
            }
        }
    }

    fn parse(full_message: &String) -> Result<(MessageCommands, String), GlobalError> {
        let reg = Regex::new(r"^(![A-Z]+)\s(.*)")?;
        let captures = reg.captures(full_message).ok_or("Failed to capture")?;
        let command = captures.get(1).ok_or("Failed to capture command")?;
        let text = captures.get(2).ok_or("Failed to capture text")?;
        match command.as_str().to_uppercase().as_str() {
            "!PROMPT" => Ok((MessageCommands::StoreChatMessage, text.as_str().to_string())),
            _ => Err(anyhow!("Error parsing message {}", full_message).into()),
        }
    }

    async fn if_exists(&self) -> anyhow::Result<bool> {
        let query = "SELECT * FROM chat_messages WHERE text = $1";
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let rows = client.query(query, &[&self.text]).await?;
        Ok(rows.len() > 0)
    }

    pub async fn verify_and_send(&self) -> Result<(), GlobalError> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        info!("Got message: {:?}", self);
        match &self.command {
            MessageCommands::StoreChatMessage => {
                if !self.if_exists().await? {
                    let query = "INSERT INTO chat_messages ( username, text, command, status ) VALUES ($1, $2, $3, $4)";
                    client
                        .query(
                            query,
                            &[
                                &self.username,
                                &self.text,
                                &self.command.to_string(),
                                &MessageStatus::Awaiting.to_string(),
                            ],
                        )
                        .await?;
                }
            }
            MessageCommands::Unknown => {
                info!("Skipping message {:?}", self);
            }
        };

        Ok(())
    }

    pub async fn update_status(&self, status: MessageStatus) -> anyhow::Result<()> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let query = "UPDATE chat_messages SET status = $1 WHERE id = $2";
        let id = Uuid::from_str(self.id.as_ref().unwrap())?;
        client.query(query, &[&status.to_string(), &id]).await?;
        Ok(())
    }
}
