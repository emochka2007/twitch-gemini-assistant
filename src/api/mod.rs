use crate::pg::pg::PgConnect;
use crate::twitch::chat_message::ChatMessage;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use std::fs;
use time::format_description::parse;
use tracing::error;

#[derive(Serialize)]
struct ChatMessagesResponse {
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct BulkUpdateRequest {
    ids: Vec<String>,
}

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
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

#[post("/update")]
async fn echo(req: web::Json<BulkUpdateRequest>) -> impl Responder {
    // ChatMessage::bulk_update(&req.ids)
    //     .await
    //     .unwrap_or_else(|e| error!("{:?}", e));
    web::Json(SuccessResponse { success: true })
}

pub async fn run_server() -> std::io::Result<()> {
    println!("Starting server at http://localhost:8080");

    HttpServer::new(|| App::new().service(hello).service(echo).service(paths))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
