use crate::twitch::TwitchApi;
mod pg;
use crate::event_poller::EventPoller;
use crate::pg::pg::PgConnect;

mod donation_alerts;
mod event_poller;
mod prompt;
mod terminal;
mod twitch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("Env file is not loaded into the project");
    // fetch_latest_donations().await?;
    let pool = PgConnect::create_pool_from_env()?;
    let client = pool.get().await?;
    PgConnect::run_migrations(&client).await?;
    tokio::spawn(async {
        EventPoller::init().await.unwrap();
    });
    TwitchApi::listen_to_chat().await?;

    Ok(())
}
