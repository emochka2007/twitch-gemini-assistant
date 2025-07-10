use crate::pg::pg::PgConnect;
use anyhow::anyhow;
use regex::Regex;
use serde::__private::de::IdentifierDeserializer;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use tokio_postgres::Row;
use tracing::{error, info};
use tracing_subscriber::field::display::Messages;
use uuid::Uuid;

type GlobalError = Box<dyn Error>;
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    id: Option<String>,
    pub command: MessageCommands,
    pub(crate) text: String,
    pub(crate) username: String,
    status: MessageStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum MessageStatus {
    Unverified,
    Awaiting,
    InProcess,
    Completed,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum MessageCommands {
    StoreChatMessage,
    SetTheme,
    SetSong,
    Unknown,
}
impl Display for MessageCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageCommands::StoreChatMessage => "!STORE".to_string(),
            MessageCommands::SetTheme => "!SET".to_string(),
            MessageCommands::SetSong => "!PLAY".to_string(),
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
            "!SET" => MessageCommands::SetTheme,
            "!PLAY" => MessageCommands::SetSong,
            _ => MessageCommands::Unknown,
        };
        Ok(command)
    }
}
impl FromStr for MessageStatus {
    type Err = GlobalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let status = match s {
            "UNVERIFIED" => MessageStatus::Unverified,
            "AWAITING" => MessageStatus::Awaiting,
            "COMPLETED" => MessageStatus::Completed,
            "IN_PROCESS" => MessageStatus::InProcess,
            _ => {
                return Err(anyhow!("Error from str for MessageStatus").into_boxed_dyn_error());
            }
        };
        Ok(status)
    }
}

impl Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MessageStatus::Unverified => "UNVERIFIED".to_string(),
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
            "!SET" => Ok((MessageCommands::SetTheme, text.as_str().to_string())),
            "!PLAY" => Ok((MessageCommands::SetSong, text.as_str().to_string())),
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

    async fn is_duplicate_theme(&self) -> anyhow::Result<bool> {
        let query = "SELECT * FROM chat_messages WHERE username = $1 and status = $2";
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let rows = client
            .query(query, &[&self.username, &self.status.to_string()])
            .await?;
        Ok(rows.len() > 0)
    }

    pub async fn insert(&self) -> Result<(), GlobalError> {
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
                                &self.text.trim(),
                                &self.command.to_string().as_str().trim(),
                                &MessageStatus::Unverified.to_string(),
                            ],
                        )
                        .await?;
                }
            }
            MessageCommands::SetTheme => {
                if !self.is_duplicate_theme().await? {
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
            MessageCommands::SetSong => {
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

    fn row_to_chat_message(row: &Row) -> anyhow::Result<ChatMessage> {
        let id: Uuid = row.try_get("id")?;
        let command: String = row.try_get("command")?;
        let text: String = row.try_get("text")?;
        let username: String = row.try_get("username")?;
        let status: String = row.try_get("status")?;
        let msg_status = MessageStatus::from_str(&status).unwrap();
        Ok(ChatMessage::new(
            Some(String::from(id)),
            text,
            MessageCommands::from_str(&command)?,
            username,
            msg_status,
        ))
    }

    pub async fn get_all_unverified() -> anyhow::Result<Vec<ChatMessage>> {
        let query = "SELECT * FROM chat_messages WHERE status = 'UNVERIFIED'";
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let rows = client.query(query, &[]).await?;
        let chat_messages: Vec<ChatMessage> = rows
            .into_iter()
            .map(|row| Self::row_to_chat_message(&row).unwrap())
            .collect();
        Ok(chat_messages)
    }

    pub async fn bulk_update(ids: &Vec<String>) -> anyhow::Result<()> {
        for id in ids {
            let query = "UPDATE chat_messages SET status = $1 WHERE status = $2 and id = $3";
            let pool = PgConnect::create_pool_from_env()?;
            let client = pool.get().await?;
            let id = Uuid::from_str(id)?;
            client
                .query(
                    query,
                    &[
                        &MessageStatus::Awaiting.to_string(),
                        &MessageStatus::Unverified.to_string(),
                        &id,
                    ],
                )
                .await?;
        }
        Ok(())
    }
}
