use crate::api::website_config::WebsiteConfig;
use crate::open_ai::OpenAI;
use crate::open_ai::types::ApiMessage;
use crate::pg::pg::PgConnect;
use crate::twitch::chat_message::{ChatMessage, MessageStatus};
use actix_web::{App, HttpServer, Responder, get, post, web};
use serde::de::Unexpected::Str;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

pub mod website_config;

#[derive(Serialize)]
struct ChatMessagesResponse {
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct BulkUpdateRequest {
    ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct GetMessage {
    messages: Vec<ApiMessage>,
}

#[derive(Deserialize)]
struct UpdateConfig {
    sound_name: String,
    theme: String,
    alert: String,
}

#[derive(Deserialize)]
struct UpdatePrompt {
    prompt: String,
}

#[derive(Deserialize)]
struct UpdateEraseMessages {
    status: bool,
}
#[derive(Deserialize)]
struct AddAdminMessage {
    text: String,
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
    prompt: String,
    erase_message: bool,
    admin_message: Option<String>,
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
    WebsiteConfig::update_config(&req.sound_name, &req.theme, &req.alert)
        .await
        .unwrap();
    web::Json(SuccessResponse { success: true })
}

#[post("/update-prompt")]
async fn update_prompt(req: web::Json<UpdatePrompt>) -> impl Responder {
    WebsiteConfig::update_prompt(&req.prompt).await.unwrap();
    web::Json(SuccessResponse { success: true })
}

#[post("/update-erase-messages")]
async fn update_erase_messages(req: web::Json<UpdateEraseMessages>) -> impl Responder {
    WebsiteConfig::update_erase_message(&req.status)
        .await
        .unwrap();
    web::Json(SuccessResponse { success: true })
}

#[post("/add-admin-message")]
async fn add_admin_message(req: web::Json<AddAdminMessage>) -> impl Responder {
    let pool = PgConnect::create_pool_from_env().unwrap();
    let client = pool.get().await.unwrap();
    let query =
        "INSERT INTO chat_messages ( username, text, command, status ) VALUES ($1, $2, $3, $4)";
    client
        .query(
            query,
            &[
                &"admin",
                &req.text.trim(),
                &"ADMIN",
                &MessageStatus::Awaiting.to_string(),
            ],
        )
        .await
        .unwrap();
    web::Json(SuccessResponse { success: true })
}

#[post("/chat")]
async fn get_message(mut req: web::Json<GetMessage>) -> impl Responder {
    info!("{:?}", req);
    loop {
        let rand = rand::random_range(0..100);
        if rand >= 71 {
            let open_ai = OpenAI::new().unwrap();
            let response = open_ai.send_user_message(&mut req.messages).await.unwrap();
            return web::Json(ChatResponse { response });
        } else {
            match ChatMessage::get_user_chat_message().await {
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
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[get("/config")]
async fn get_config() -> impl Responder {
    let config = WebsiteConfig::get_config().await;
    if let Ok(admin_message) = ChatMessage::get_admin_message().await {
        admin_message
            .update_status(MessageStatus::Completed)
            .await
            .unwrap();
        return web::Json(ConfigResponse {
            sound: config.sound_name,
            theme: config.theme,
            alert: config.alert,
            prompt: config.prompt,
            erase_message: config.erase_message,
            admin_message: Some(admin_message.text),
        });
    };
    web::Json(ConfigResponse {
        sound: config.sound_name,
        theme: config.theme,
        alert: config.alert,
        prompt: config.prompt,
        erase_message: config.erase_message,
        admin_message: None,
    })
}

pub async fn run_server() -> std::io::Result<()> {
    info!("Starting server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(get_message)
            .service(get_config)
            .service(update_prompt)
            .service(update_config)
            .service(update_erase_messages)
            .service(add_admin_message)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
