mod prompt;
pub mod types;

use crate::api::website_config::WebsiteConfig;
use crate::open_ai::prompt::get_system_prompt;
use crate::open_ai::types::{ApiMessage, ChatCompletionResponse};
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::io::ErrorKind;
use std::{env, io};
use tracing::{error, info};

pub struct OpenAI {
    key: String,
    client: Client,
    base_url: String,
}
impl OpenAI {
    pub fn new() -> Result<Self> {
        let client = Client::new();
        let base_url = "https://api.openai.com/v1/chat/completions";

        let open_ai_token = env::var("OPEN_AI_KEY")?;

        Ok(Self {
            key: open_ai_token,
            client,
            base_url: base_url.to_string(),
        })
    }

    async fn post<T: DeserializeOwned>(&self, body: Value) -> Result<T> {
        let response = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.key))
            .json(&body)
            .send()
            .await?;
        match response.json::<T>().await {
            Ok(response) => Ok(response),
            Err(e) => {
                error!("{e:?}");
                Err(anyhow!("Error in post"))
            }
        }
    }

    fn parse_choice(chat_completion_response: &ChatCompletionResponse) -> Result<String> {
        match chat_completion_response.choices.first() {
            Some(choice) => Ok(choice.message.content.to_string()),
            None => Err(io::Error::new(ErrorKind::InvalidData, "Choice not found").into()),
        }
    }

    pub async fn send_user_message(&self, messages: &mut Vec<ApiMessage>) -> Result<String> {
        Self::keep_last_n(messages, 20);
        let config = WebsiteConfig::get_config().await;
        messages.insert(
            0,
            ApiMessage {
                role: "system".to_string(),
                content: config.prompt,
            },
        );
        info!("{:?}", messages);
        let body = json!({
        "model": "gpt-4o",
        "store": true,
        "messages": messages
        });
        let response = self.post::<ChatCompletionResponse>(body).await?;
        Self::parse_choice(&response)
    }

    fn keep_last_n<T>(v: &mut Vec<T>, n: usize) {
        if v.len() > n {
            v.drain(0..v.len() - n);
        }
    }
}
