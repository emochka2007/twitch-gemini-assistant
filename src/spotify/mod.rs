use crate::terminal::send_to_terminal;
use reqwest::Client;
use rspotify::clients::OAuthClient;
use rspotify::model::{PlayableId, TrackId};
use rspotify::{AuthCodeSpotify, Config, Credentials, OAuth, scopes};
use tracing::error;

pub fn extract_track_id(url: &str) -> Option<&str> {
    url.split("/track/")
        .nth(1)
        .and_then(|s| s.split('?').next())
}

pub async fn get_spotify_auth_token(url: &str) -> anyhow::Result<()> {
    match extract_track_id(url) {
        Some(uri) => {
            let creds = Credentials::from_env().unwrap();
            let scopes = scopes!("user-modify-playback-state", "user-read-playback-state");
            let oauth = OAuth::from_env(scopes).unwrap();
            // Cache token to disk so you only need to log in once
            let config = Config {
                token_cached: true,
                ..Default::default()
            };

            let mut spotify = AuthCodeSpotify::with_config(creds, oauth, config);

            // 1. Get the authorization URL and have the user visit it
            let auth_url = spotify.get_authorize_url(false)?;
            println!("Open this URL in your browser:\n{auth_url}");

            // 2. Either run `spotify.prompt_for_token(&auth_url).await?;` (requires `cli` feature)
            //    or copy the redirect URL you get redirected to and paste it here:
            let code = spotify
                .prompt_for_token(&auth_url)
                .await
                .expect("could not get token");

            // Now `spotify.token` is set and you can call the Player API.
            // Replace with any valid track URI:
            let track_uri = format!("spotify:track:{}", uri);
            let track_id = TrackId::from_uri(&track_uri).unwrap();

            // Enqueue it on your currently active device:
            spotify
                .add_item_to_queue(PlayableId::Track(track_id), None)
                .await
                .expect("failed to add to queue");
        }
        None => {
            error!("Incorrect url {:?}", url);
        }
    }
    Ok(())
}

pub async fn open_track(url: &str) -> anyhow::Result<()> {
    if let Some(track_id) = extract_track_id(url) {
        let command = format!("open \"spotify:track:{}\"", track_id);
        send_to_terminal(&command).await?;
    } else {
        eprintln!("Track ID not found.");
    }
    Ok(())
}
