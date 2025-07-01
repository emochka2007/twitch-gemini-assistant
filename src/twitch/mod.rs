pub mod chat_message;
use crate::twitch::chat_message::ChatMessage;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fmt::Debug;
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage::Privmsg;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

pub struct TwitchApi {
    secret: String,
    client: String,
    access_token: Option<String>,
}

impl TwitchApi {
    pub async fn listen_to_chat() -> Result<(), Box<dyn Error>> {
        // default configuration is to join chat as anonymous.
        let config = ClientConfig::default();
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        // first thing you should do: start consuming incoming messages,
        // otherwise they will back up.
        let join_handle: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> =
            tokio::spawn(async move {
                while let Some(message) = incoming_messages.recv().await {
                    if let Privmsg(priv_msg) = message {
                        let msg_id_tag = priv_msg.source.tags.0.get("msg-id");
                        match msg_id_tag {
                            Some(_) => {
                                if let Ok(chat_message) = ChatMessage::from_raw_message(
                                    priv_msg.message_text,
                                    priv_msg.sender.login,
                                ) {
                                    chat_message.verify_and_send().await.unwrap();
                                }
                            }
                            None => (),
                        }
                    }
                }
                Ok(())
            });

        // join a channel
        // This function only returns an error if the passed channel login name is malformed,
        // so in this simple case where the channel name is hardcoded we can ignore the potential
        // error with `unwrap`.
        let streamer_channel = env::var("STREAMER").expect("STREAMER env is not set");
        client.join(streamer_channel.to_owned()).unwrap();

        // keep the tokio executor alive.
        // If you return instead of waiting the background task will exit.
        join_handle.await?.expect("Error in join handle");
        Ok(())
    }
}
