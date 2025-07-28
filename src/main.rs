use crate::api::{chat, get_config, update_config};
use actix_web::web::ServiceConfig;
use shuttle_actix_web::ShuttleActixWeb;

mod pg;
use crate::pg::pg::PgConnect;

mod api;
mod chaos;
mod event_poller;
mod open_ai;
mod prompt;
mod spotify;
mod terminal;
mod twitch;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("Env file is not loaded into the project");
    let pool = PgConnect::create_pool_from_env().unwrap();
    let client = pool.get().await.unwrap();
    PgConnect::run_migrations(&client).await.unwrap();
    // tokio::spawn(async {
    //     EventPoller::init().await.unwrap();
    // });
    // TwitchApi::listen_to_chat().await.unwrap();
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(chat).service(get_config).service(update_config);
    };

    Ok(config.into())
}
