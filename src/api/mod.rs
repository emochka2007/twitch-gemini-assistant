pub mod website_config;

use crate::api::website_config::WebsiteConfig;
use crate::open_ai::OpenAI;
use crate::open_ai::types::ApiMessage;
use crate::twitch::chat_message::{ChatMessage, MessageStatus};
use actix_web::web::{ServiceConfig, get};
use actix_web::{App, HttpServer, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use shuttle_actix_web::ShuttleActixWeb;
use std::time::Duration;
use tracing::error;

#[derive(Serialize)]
struct ChatMessagesResponse {
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct BulkUpdateRequest {
    ids: Vec<String>,
}

#[derive(Deserialize)]
struct GetMessage {
    messages: Vec<ApiMessage>,
}

#[derive(Deserialize)]
struct UpdateConfig {
    sound_name: Option<String>,
    theme: Option<String>,
    alert: Option<String>,
}

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
}

#[derive(Serialize)]
struct ConfigResponse {
    sound: String,
    theme: String,
    alert: String,
}

#[get("/all")]
async fn hello() -> impl Responder {
    let messages = ChatMessage::get_all_unverified().await.unwrap();
    web::Json(ChatMessagesResponse { messages })
}

#[get("/paths")]
async fn paths() -> impl Responder {
    let dir_path = "/Users/nikitapoznyakov/Desktop/chat-tvari";
    let mut paths = Vec::new();
    let mut entries = tokio::fs::read_dir(dir_path).await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                paths.push(name.to_string().replace(".css", ""));
            }
        }
    }
    web::Json(paths)
}

#[post("/update-config")]
async fn update_config(req: web::Json<UpdateConfig>) -> impl Responder {
    let default = String::from("none");
    WebsiteConfig::update_config(WebsiteConfig {
        sound_name: req.sound_name.clone().unwrap_or(default.clone()),
        theme: req.theme.clone().unwrap_or(default.clone()),
        alert: req.alert.clone().unwrap_or(default),
    })
    .await
    .unwrap();
    web::Json(SuccessResponse { success: true })
}

#[post("/chat")]
pub async fn chat(mut req: web::Json<GetMessage>) -> impl Responder {
    loop {
        let rand = rand::random_range(0..100);
        if rand >= 31 {
            let open_ai = OpenAI::new().unwrap();
            let response = open_ai.send_user_message(&mut req.messages).await.unwrap();
            return web::Json(ChatResponse { response });
        }

        match ChatMessage::get_ai_chat_message().await {
            Ok(message) => {
                message
                    .update_status(MessageStatus::Completed)
                    .await
                    .unwrap();
                return web::Json(ChatResponse {
                    response: message.text,
                });
            }
            Err(e) => {
                error!("Error getting chat message {e:?}");
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[get("/config")]
async fn get_config() -> impl Responder {
    let config = WebsiteConfig::get_config().await;
    web::Json(ConfigResponse {
        sound: config.sound_name,
        theme: config.theme,
        alert: config.alert,
    })
}
