use crate::command_framework::command_framework::CommandFramework;
use crate::commands::chaos::ChaosCommand;
use crate::commands::play::PlayCommand;
// use crate::api::run_server;
// use crate::twitch::TwitchApi;
mod pg;
// use crate::event_poller::EventPoller;
use crate::pg::pg::PgConnect;
use crate::twitch::{TwitchApi, TwitchApiCredentials};

// mod api;
mod chaos;
// mod event_poller;
mod command_framework;
mod commands;
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
    // tokio::spawn(async {});
    // TwitchApi::listen_to_chat().await.unwrap();
    // run_server().await.unwrap();

    let twitch_api = TwitchApi::builder()
        .credentials(TwitchApiCredentials::new(
            "".to_owned(),
            "".to_owned(),
            Some("".to_owned()),
        ))
        .framework(CommandFramework::new(vec![
            Box::new(PlayCommand::new(PgConnect::create_pool_from_env()?)),
            Box::new(ChaosCommand::new(PgConnect::create_pool_from_env()?)),
        ]))
        .build();

    twitch_api.run().await?;

    Ok(())
}
