use crate::twitch::TwitchApi;
use tracing::{error, info};
mod pg;
use crate::event_poller::EventPoller;
use crate::pg::pg::PgConnect;
use crate::prompt::get_formatted_prompt;
use crate::terminal::test_send_to_terminal;

mod event_poller;
mod prompt;
mod terminal;
mod twitch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("Env file is not loaded into the project");
    let prompt = get_formatted_prompt("Can u edit main.rs and add small add function there");
    match test_send_to_terminal(&prompt).await {
        Ok(_) => {
            info!("Completed");
        }
        Err(error) => {
            error!("Gemini child error {:?}", error);
        }
    }
    // let pool = PgConnect::create_pool_from_env()?;
    // let client = pool.get().await?;
    // PgConnect::run_migrations(&client).await?;
    // tokio::spawn(async move {
    //     EventPoller::init().await.unwrap();
    // });
    // TwitchApi::listen_to_chat().await?;

    Ok(())
}
