use crate::command_framework::{Context, Event, Framework, Message, User};
use crate::twitch::TwitchApiCredentials;
use crate::twitch::builder::TwitchApiBuilder;
use std::env;
use tokio::task::JoinHandle;
use tracing::{info, instrument};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage::Privmsg;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

#[derive(Debug)]
pub struct TwitchApi {
    credentials: TwitchApiCredentials,
    framework: Option<Box<dyn Framework>>,
}

impl TwitchApi {
    pub fn new(
        credentials: TwitchApiCredentials,
        framework: Option<Box<dyn Framework>>,
    ) -> TwitchApi {
        Self {
            credentials,
            framework,
        }
    }

    pub fn builder() -> TwitchApiBuilder {
        TwitchApiBuilder::default()
    }

    #[instrument]
    pub async fn run(self) -> anyhow::Result<()> {
        let config = ClientConfig::default();
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        let join_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                if let Privmsg(priv_msg) = message {
                    info!("Received privmsg: {:?}", priv_msg);

                    if let Some(framework) = self.framework.as_ref() {
                        let ctx = Context {};
                        let user = User {
                            id: priv_msg.sender.id,
                            login: priv_msg.sender.login,
                            name: priv_msg.sender.name,
                        };
                        let message = Message {
                            id: priv_msg.message_id,
                            user,
                            content: priv_msg.message_text,
                        };
                        let event = Event::Message(message);

                        framework.dispatch(ctx, event).await;
                    }
                }
            }
            Ok(())
        });

        let streamer_channel = env::var("STREAMER").expect("STREAMER env is not set");
        client.join(streamer_channel.to_owned())?;

        join_handle.await?.expect("Error in join handle");
        Ok(())
    }
}
