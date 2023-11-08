use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Error as ReqwestError;
use serde_json::{json, Value};
use std::env;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
impl StdError for LlmError {}

lazy_static! {
    static ref AI_MODEL: String = {
        dotenvy::dotenv().ok();
        dotenvy::var("AI_MODEL").unwrap_or_else(|_| panic!("AI_MODEL must be set"))
    };
}

#[derive(Debug)]
pub enum LlmError {
    MissingApiKey,
    MissingContent,
    ReqwestError(ReqwestError),
}

impl From<ReqwestError> for LlmError {
    fn from(error: ReqwestError) -> Self {
        LlmError::ReqwestError(error)
    }
}

impl Display for LlmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LlmError::MissingApiKey => write!(f, "Missing OpenAI API Key"),
            LlmError::MissingContent => write!(f, "Missing content in LLM response"),
            LlmError::ReqwestError(error) => write!(f, "Reqwest error: {}", error),
        }
    }
}

pub async fn get_llm_response(
    system_message: &str,
    user_message: &str,
) -> Result<String, LlmError> {
    let api_key = env::var("OPENAI_API_KEY").map_err(|_| LlmError::MissingApiKey)?;
    let chat_url = "https://api.openai.com/v1/chat/completions";

    let messages = json!([
        {
            "role": "system",
            "content": system_message
        },
        {
            "role": "user",
            "content": user_message
        }
    ]);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|_| LlmError::MissingApiKey)?,
    );

    println!("Request to LLM");
    let client = reqwest::Client::new();
    let ai_model: String = AI_MODEL.to_string();
    let res: Value = client
        .post(chat_url)
        .headers(headers)
        .json(&json!({
            "model": ai_model,
            "messages": messages
        }))
        .send()
        .await?
        .json()
        .await?;

    if let Some(content) = res["choices"][0]["message"]["content"].as_str() {
        Ok(content.to_string())
    } else {
        println!("LLM response: {:?}", res);
        Err(LlmError::MissingContent)
    }
}
