use serde::{Serialize, Deserialize};

// LLM Provider enum
#[derive(Debug, Clone, PartialEq)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Custom(String),
}

// Configuration for LLM requests
#[derive(Debug, Clone)]
pub struct LlmRequestConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub api_key: String,
    pub api_endpoint: Option<String>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub timeout_seconds: u64,
}

impl Default for LlmRequestConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            api_key: String::new(),
            api_endpoint: None,
            temperature: 0.7,
            max_tokens: Some(1024),
            timeout_seconds: 60,
        }
    }
}

// OpenAI request structures
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
}

// Anthropic request structures
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub prompt: String,
    pub temperature: f32,
    pub max_tokens_to_sample: u32,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub completion: String,
}
