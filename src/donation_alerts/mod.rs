mod auth;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Donation {
    pub username: String,
    pub amount: f64,
    pub message: Option<String>,
    pub created_at: String,
}
#[derive(Debug, Deserialize)]
struct DonationResponse {
    data: Vec<Donation>,
}

pub async fn fetch_latest_donations() -> Result<(), Box<dyn Error>> {
    Ok(auth::fetch_latest_donations().await)
}
