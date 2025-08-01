use crate::command_framework::Framework;
use crate::twitch::TwitchApiCredentials;
use crate::twitch::api::TwitchApi;

pub struct TwitchApiBuilder {
    credentials: Option<TwitchApiCredentials>,
    framework: Option<Box<dyn Framework>>,
}

impl Default for TwitchApiBuilder {
    fn default() -> Self {
        Self {
            credentials: None,
            framework: None,
        }
    }
}

impl TwitchApiBuilder {
    pub fn credentials(mut self, credentials: TwitchApiCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn framework<F>(mut self, framework: F) -> Self
    where
        F: Framework + 'static,
    {
        self.framework = Some(Box::new(framework));
        self
    }

    pub fn build(self) -> TwitchApi {
        let credentials = self
            .credentials
            .expect("No Twitch API credentials provided.");
        TwitchApi::new(credentials, self.framework)
    }
}
