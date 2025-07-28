use crate::api::run_server;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("Env file is not loaded into the project");
    let pool = PgConnect::create_pool_from_env()?;
    let client = pool.get().await?;
    PgConnect::run_migrations(&client).await?;
    // tokio::spawn(async {
    //     EventPoller::init().await.unwrap();
    // });
    // TwitchApi::listen_to_chat().await.unwrap();
    run_server().await.unwrap();
    Ok(())
}
