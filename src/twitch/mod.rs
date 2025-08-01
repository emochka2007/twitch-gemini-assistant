mod api;
mod builder;
pub mod chat_message;

#[derive(Debug)]
pub struct TwitchApiCredentials {
    secret: String,
    client: String,
    access_token: Option<String>,
}

impl TwitchApiCredentials {
    pub fn new(secret: String, client: String, access_token: Option<String>) -> Self {
        Self {
            secret,
            client,
            access_token,
        }
    }
}

pub use api::TwitchApi;
