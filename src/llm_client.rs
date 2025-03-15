//! LLM Client for interacting with various language model APIs
//! 
//! This module provides a unified interface for sending prompts to
//! different LLM providers like OpenAI, Anthropic, etc.

use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Result, anyhow};

/// Supported LLM providers
#[derive(Debug, Clone)]
pub enum LlmProvider {
    /// OpenAI API (ChatGPT, GPT-4, etc.)
    OpenAI,
    /// Anthropic API (Claude, etc.)
    Anthropic,
    /// Custom API endpoint
    Custom(String),
}

/// Configuration for LLM API requests
#[derive(Debug, Clone)]
pub struct LlmRequestConfig {
    /// The LLM provider to use
    pub provider: LlmProvider,
    /// The model name to use (e.g., "gpt-3.5-turbo", "claude-2")
    pub model: String,
    /// Temperature setting (0.0 to 1.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// API key for authentication
    pub api_key: String,
    /// Optional custom API endpoint
    pub api_endpoint: Option<String>,
}

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

// OpenAI API types
#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct OpenAIMessage2 {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage2,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

// Anthropic API types
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    prompt: String,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens_to_sample: Option<u32>,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    completion: String,
}

/// Client for interacting with LLM APIs
pub struct LlmClient {
    http_client: Client,
    pub config: LlmRequestConfig,
}

impl LlmClient {
    /// Create a new LLM client with the given configuration
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
    
    /// Create an LLM client from block modifiers
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
                "model" => {
                    config.model = value.clone();
                },
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
                "api_key" => {
                    config.api_key = value.clone();
                },
                "api_endpoint" => {
                    config.api_endpoint = Some(value.clone());
                },
                _ => {} // Ignore unknown modifiers
            }
        }
        
        Self::new(config)
    }
    
    /// Send a prompt to the configured LLM provider and return the response
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
        
        let response = self.http_client.post(&endpoint)
            .header(header::CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }
        
        let response_data: OpenAIResponse = response.json().await?;
        if response_data.choices.is_empty() {
            return Err(anyhow!("OpenAI API returned no choices"));
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
        
        let response = self.http_client.post(&endpoint)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-API-Key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {}", error_text));
        }
        
        let response_data: AnthropicResponse = response.json().await?;
        Ok(response_data.completion)
    }
    
    async fn send_custom_prompt(&self, endpoint: &str, prompt: &str) -> Result<String> {
        // Simple implementation for custom endpoints
        // In a real implementation, this would be more configurable
        
        let request = serde_json::json!({
            "prompt": prompt,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });
        
        let response = self.http_client.post(endpoint)
            .header(header::CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Custom API error: {}", error_text));
        }
        
        // For custom endpoints, we'll just return the raw text
        // In a real implementation, this would parse the response based on configuration
        let response_text = response.text().await?;
        Ok(response_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_from_block_modifiers() {
        let modifiers = vec![
            ("provider".to_string(), "openai".to_string()),
            ("model".to_string(), "gpt-4".to_string()),
            ("temperature".to_string(), "0.5".to_string()),
            ("max_tokens".to_string(), "2048".to_string()),
        ];
        
        let client = LlmClient::from_block_modifiers(&modifiers);
        
        match client.config.provider {
            LlmProvider::OpenAI => {},
            _ => panic!("Expected OpenAI provider"),
        }
        
        assert_eq!(client.config.model, "gpt-4");
        assert_eq!(client.config.temperature, 0.5);
        assert_eq!(client.config.max_tokens, Some(2048));
    }
}
