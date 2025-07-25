use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    pub choices: Vec<Choice>,
    usage: Option<Usage>,
    service_tier: String,
    system_fingerprint: String,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    index: i32,
    pub message: Message,
    logprobs: Option<serde_json::Value>, // Use serde_json::Value for fields that can vary or are optional
    finish_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    role: String,
    pub content: String,
    refusal: Option<serde_json::Value>, // assuming refusal can be null or some other structure
}

#[derive(Deserialize, Debug)]
struct TokenDetails {
    cached_tokens: Option<i32>,
    audio_tokens: Option<i32>,
    reasoning_tokens: Option<i32>,
    accepted_prediction_tokens: Option<i32>,
    rejected_prediction_tokens: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct OutputItem {
    pub id: String,
    #[serde(rename = "type")]
    pub output_type: String,
    pub status: String,
    pub content: Vec<ContentItem>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub annotations: Vec<serde_json::Value>,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct Reasoning {
    pub effort: Option<serde_json::Value>,
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TextField {
    pub format: TextFormat,
}

#[derive(Debug, Deserialize)]
pub struct TextFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    // pub input_tokens: u32,
    // pub output_tokens: u32,
    // pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct InputTokensDetails {
    pub cached_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct OutputTokensDetails {
    pub reasoning_tokens: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ApiMessage {
    // system, user
    pub(crate) role: String,
    pub(crate) content: String,
}
