use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;
use warp::Filter;

const CLIENT_ID: &str = "";
const CLIENT_SECRET: &str = "";
const REDIRECT_URI: &str = "http://localhost:8080/callback";
const AUTH_URL: &str = "https://www.donationalerts.com/oauth/authorize";
const TOKEN_URL: &str = "https://www.donationalerts.com/oauth/token";
const SCOPES: &str = "oauth-user-show oauth-donation-index";

// ========== Step 1: Build and show auth URL ==========
fn build_authorize_url() -> String {
    let mut url = Url::parse(AUTH_URL).unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPES);
    url.into_string()
}

// ========== Step 2: Set up tiny server to capture code ==========
async fn wait_for_code() -> String {
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();

    let route = warp::get()
        .and(warp::path("callback"))
        .and(warp::query::<HashMap<String, String>>())
        .then(move |params: HashMap<String, String>| async move {
            let mut tx = Some(tx);
            if let Some(code) = params.get("code") {
                if let Some(tx) = tx {
                    let _ = tx.send(code.clone());
                }
                warp::reply::html("Authorization successful. You can close this window.")
            } else {
                warp::reply::html("Missing code in query string.")
            }
        });

    tokio::spawn(warp::serve(route).run(([127, 0, 0, 1], 8080)));

    rx.await.expect("Failed to receive code")
}

// ========== Step 3: Exchange code for access token ==========
#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
}

async fn exchange_code_for_token(code: &str) -> Result<TokenResponse, reqwest::Error> {
    let client = Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("redirect_uri", REDIRECT_URI),
        ("code", code),
    ];

    let res = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    Ok(res)
}

// ========== Step 4: Fetch user donations ==========
#[derive(Debug, Deserialize)]
struct Donation {
    username: String,
    amount: f64,
    message: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct DonationList {
    data: Vec<Donation>,
}

async fn fetch_donations(access_token: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let response = client
        .get("https://www.donationalerts.com/api/v1/alerts/donations")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<DonationList>()
        .await?;

    println!("Donations: {:#?}", response.data);
    Ok(())
}

// ========== Step 5: Refresh token ==========
async fn refresh_token(refresh_token: &str) -> Result<TokenResponse, reqwest::Error> {
    let client = Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("scope", SCOPES),
    ];

    let res = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    Ok(res)
}

pub async fn fetch_latest_donations() {
    println!("1. Go to this URL to authorize:");
    println!("{}", build_authorize_url());

    let code = wait_for_code().await;
    println!("Received code: {}", code);

    let token_response = exchange_code_for_token(&code)
        .await
        .expect("Failed to get access token");

    println!("Access Token: {}", token_response.access_token);
    println!("Refresh Token: {}", token_response.refresh_token);

    // Use access token
    fetch_donations(&token_response.access_token)
        .await
        .expect("Failed to fetch donations");

    // Optional: Refresh token
    // let refreshed = refresh_token(&token_response.refresh_token).await.unwrap();
    // println!("New access token: {}", refreshed.access_token);
}
