use anyhow::Result;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// LLM Provider types
#[derive(Debug, Clone)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Custom(String),
}

// LLM Request configuration
#[derive(Debug, Clone)]
pub struct LlmRequestConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub timeout_seconds: u64,
    pub api_key: String,
    pub api_endpoint: Option<String>,
}

// Default configuration
impl Default for LlmRequestConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: Some(1024),
            timeout_seconds: 30,
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            api_endpoint: None,
        }
    }
}

// OpenAI API structures
#[derive(Serialize, Debug)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

// Anthropic API structures
#[derive(Serialize, Debug)]
struct AnthropicRequest {
    model: String,
    prompt: String,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens_to_sample: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct AnthropicResponse {
    completion: String,
}

// LLM Client
pub struct LlmClient {
    http_client: Client,
    pub config: LlmRequestConfig,
}

impl LlmClient {
    pub fn new(config: LlmRequestConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            http_client,
            config,
        }
    }
    
    pub async fn send_prompt(&self, prompt: &str) -> Result<String> {
        match self.config.provider {
            LlmProvider::OpenAI => self.send_openai_prompt(prompt).await,
            LlmProvider::Anthropic => self.send_anthropic_prompt(prompt).await,
            LlmProvider::Custom(ref endpoint) => self.send_custom_prompt(endpoint, prompt).await,
        }
    }
    
    async fn send_openai_prompt(&self, prompt: &str) -> Result<String> {
        let endpoint = self.config.api_endpoint.clone()
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
            
        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        
        let response = self.http_client
            .post(&endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("OpenAI API error: {}", error_text);
        }
        
        let response_data: OpenAIResponse = response.json().await?;
        if response_data.choices.is_empty() {
            anyhow::bail!("OpenAI API returned no choices");
        }
        
        Ok(response_data.choices[0].message.content.clone())
    }
    
    async fn send_anthropic_prompt(&self, prompt: &str) -> Result<String> {
        let endpoint = self.config.api_endpoint.clone()
            .unwrap_or_else(|| "https://api.anthropic.com/v1/complete".to_string());
            
        // Format prompt for Claude
        let formatted_prompt = format!("\n\nHuman: {}\n\nAssistant:", prompt);
        
        let request = AnthropicRequest {
            model: self.config.model.clone(),
            prompt: formatted_prompt,
            temperature: self.config.temperature,
            max_tokens_to_sample: self.config.max_tokens,
        };
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "X-API-Key",
            header::HeaderValue::from_str(&self.config.api_key)?,
        );
        headers.insert(
            "Anthropic-Version",
            header::HeaderValue::from_static("2023-06-01"),
        );
        
        let response = self.http_client
            .post(&endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Anthropic API error: {}", error_text);
        }
        
        let response_data: AnthropicResponse = response.json().await?;
        Ok(response_data.completion.clone())
    }
    
    async fn send_custom_prompt(&self, endpoint: &str, prompt: &str) -> Result<String> {
        // Simple implementation for custom endpoints
        // In a real implementation, this would be more configurable
        
        let request = serde_json::json!({
            "prompt": prompt,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        
        let response = self.http_client
            .post(endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Custom LLM API error: {}", error_text);
        }
        
        // For custom endpoints, we'll just return the raw response text
        // In a real implementation, this would parse the response based on the expected format
        let response_text = response.text().await?;
        Ok(response_text)
    }
    
    // Helper to create a client from block modifiers
    pub fn from_block_modifiers(modifiers: &[(String, String)]) -> Self {
        let mut config = LlmRequestConfig::default();
        
        for (key, value) in modifiers {
            match key.as_str() {
                "provider" => {
                    config.provider = match value.to_lowercase().as_str() {
                        "openai" => LlmProvider::OpenAI,
                        "anthropic" => LlmProvider::Anthropic,
                        custom => LlmProvider::Custom(custom.to_string()),
                    };
                },
                "model" => config.model = value.clone(),
                "temperature" => {
                    if let Ok(temp) = value.parse::<f32>() {
                        config.temperature = temp;
                    }
                },
                "max_tokens" => {
                    if let Ok(tokens) = value.parse::<u32>() {
                        config.max_tokens = Some(tokens);
                    }
                },
                "timeout" => {
                    if let Ok(timeout) = value.parse::<u64>() {
                        config.timeout_seconds = timeout;
                    }
                },
                "api_key" => config.api_key = value.clone(),
                "api_endpoint" => config.api_endpoint = Some(value.clone()),
                _ => {}
            }
        }
        
        // If no API key was provided in modifiers, try to get from environment
        if config.api_key.is_empty() {
            match config.provider {
                LlmProvider::OpenAI => {
                    config.api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
                },
                LlmProvider::Anthropic => {
                    config.api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
                },
                LlmProvider::Custom(_) => {
                    config.api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
                }
            }
        }
        
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    // Tests would go here
}
use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Result, anyhow};
use reqwest::{Client, header};
use serde::{Serialize, Deserialize};
use serde_json::json;

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
            max_tokens: Some(1000),
            timeout_seconds: 60,
        }
    }
}

// OpenAI request structures
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

// Anthropic request structures
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens_to_sample: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    completion: String,
}

// LLM Client implementation
pub struct LlmClient {
    http_client: Client,
    pub config: LlmRequestConfig,
}

impl LlmClient {
    pub fn new(config: LlmRequestConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            http_client,
            config,
        }
    }
    
    // Create a client from block modifiers
    pub fn from_block_modifiers(modifiers: &[(String, String)]) -> Self {
        let mut config = LlmRequestConfig::default();
        
        // Convert modifiers to a HashMap for easier lookup
        let modifiers_map: HashMap<_, _> = modifiers.iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        
        // Set provider based on modifiers
        if let Some(provider) = modifiers_map.get("provider") {
            config.provider = match *provider {
                "openai" => LlmProvider::OpenAI,
                "anthropic" => LlmProvider::Anthropic,
                custom => LlmProvider::Custom(custom.to_string()),
            };
        }
        
        // Set model if specified
        if let Some(model) = modifiers_map.get("model") {
            config.model = model.to_string();
        }
        
        // Set API key from modifiers or environment variables
        if let Some(api_key) = modifiers_map.get("api_key") {
            config.api_key = api_key.to_string();
        } else {
            // Try to get API key from environment variables
            match config.provider {
                LlmProvider::OpenAI => {
                    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                        config.api_key = key;
                    }
                },
                LlmProvider::Anthropic => {
                    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
                        config.api_key = key;
                    }
                },
                LlmProvider::Custom(_) => {
                    if let Ok(key) = std::env::var("LLM_API_KEY") {
                        config.api_key = key;
                    }
                },
            }
        }
        
        // Set API endpoint if specified
        if let Some(endpoint) = modifiers_map.get("api_endpoint") {
            config.api_endpoint = Some(endpoint.to_string());
        }
        
        // Set temperature if specified
        if let Some(temp) = modifiers_map.get("temperature") {
            if let Ok(temp_value) = temp.parse::<f32>() {
                config.temperature = temp_value;
            }
        }
        
        // Set max tokens if specified
        if let Some(max_tokens) = modifiers_map.get("max_tokens") {
            if let Ok(tokens_value) = max_tokens.parse::<u32>() {
                config.max_tokens = Some(tokens_value);
            }
        }
        
        // Set timeout if specified
        if let Some(timeout) = modifiers_map.get("timeout") {
            if let Ok(timeout_value) = timeout.parse::<u64>() {
                config.timeout_seconds = timeout_value;
            }
        }
        
        Self::new(config)
    }
    
    // Send a prompt to the LLM and get the response
    pub async fn send_prompt(&self, prompt: &str) -> Result<String> {
        match self.config.provider {
            LlmProvider::OpenAI => self.send_openai_prompt(prompt).await,
            LlmProvider::Anthropic => self.send_anthropic_prompt(prompt).await,
            LlmProvider::Custom(ref endpoint) => self.send_custom_prompt(endpoint, prompt).await,
        }
    }
    
    // Send a prompt to OpenAI
    async fn send_openai_prompt(&self, prompt: &str) -> Result<String> {
        let endpoint = self.config.api_endpoint.clone()
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
            
        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))
                .map_err(|e| anyhow!("Invalid API key: {}", e))?,
        );
        
        let response = self.http_client
            .post(&endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to OpenAI: {}", e))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }
        
        let response_data: OpenAIResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse OpenAI response: {}", e))?;
            
        if response_data.choices.is_empty() {
            return Err(anyhow!("OpenAI returned no choices"));
        }
        
        Ok(response_data.choices[0].message.content.clone())
    }
    
    // Send a prompt to Anthropic
    async fn send_anthropic_prompt(&self, prompt: &str) -> Result<String> {
        let endpoint = self.config.api_endpoint.clone()
            .unwrap_or_else(|| "https://api.anthropic.com/v1/complete".to_string());
            
        // Format prompt for Claude
        let formatted_prompt = format!("\n\nHuman: {}\n\nAssistant:", prompt);
        
        let request = AnthropicRequest {
            model: self.config.model.clone(),
            prompt: formatted_prompt,
            temperature: self.config.temperature,
            max_tokens_to_sample: self.config.max_tokens.unwrap_or(1000),
        };
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "X-API-Key",
            header::HeaderValue::from_str(&self.config.api_key)
                .map_err(|e| anyhow!("Invalid API key: {}", e))?,
        );
        
        let response = self.http_client
            .post(&endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Anthropic: {}", e))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Anthropic API error: {}", error_text));
        }
        
        let response_data: AnthropicResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse Anthropic response: {}", e))?;
            
        Ok(response_data.completion)
    }
    
    // Send a prompt to a custom endpoint
    async fn send_custom_prompt(&self, endpoint: &str, prompt: &str) -> Result<String> {
        // Simple implementation for custom endpoints
        // In a real implementation, this would be more configurable
        
        let request = serde_json::json!({
            "prompt": prompt,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        
        // Add API key if available
        if !self.config.api_key.is_empty() {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))
                    .map_err(|e| anyhow!("Invalid API key: {}", e))?,
            );
        }
        
        let response = self.http_client
            .post(endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to custom endpoint: {}", e))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Custom API error: {}", error_text));
        }
        
        // Try to extract the response from common response formats
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse custom response: {}", e))?;
            
        // Try different common response fields
        if let Some(text) = response_json.get("text").and_then(|v| v.as_str()) {
            return Ok(text.to_string());
        } else if let Some(content) = response_json.get("content").and_then(|v| v.as_str()) {
            return Ok(content.to_string());
        } else if let Some(completion) = response_json.get("completion").and_then(|v| v.as_str()) {
            return Ok(completion.to_string());
        } else if let Some(message) = response_json.get("message") {
            if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                return Ok(content.to_string());
            }
        }
        
        // If we couldn't extract a specific field, return the whole JSON as a string
        Ok(serde_json::to_string_pretty(&response_json)
            .unwrap_or_else(|_| "Failed to format response".to_string()))
    }
}
