use crate::pg::pg::PgConnect;
use crate::twitch::chat_message::ChatMessage;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use serde::{Deserialize, Serialize};
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

#[get("/")]
async fn hello() -> impl Responder {
    let messages = ChatMessage::get_all_unverified().await.unwrap();
    web::Json(ChatMessagesResponse { messages })
}

#[post("/update")]
async fn echo(req: web::Json<BulkUpdateRequest>) -> impl Responder {
    ChatMessage::bulk_update(&req.ids)
        .await
        .unwrap_or_else(|e| error!("{:?}", e));
    web::Json(SuccessResponse { success: true })
}

pub async fn run_server() -> std::io::Result<()> {
    println!("Starting server at http://localhost:8080");

    HttpServer::new(|| App::new().service(hello).service(echo))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
