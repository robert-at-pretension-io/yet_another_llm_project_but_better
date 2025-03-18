mod types;

use std::collections::HashMap;
use std::process::Command;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use serde_json::Value;
use rand::random;

pub use types::*;

// Temporary file manager for curl requests
struct TempFileManager {
    request_path: PathBuf,
    response_path: PathBuf,
}

impl TempFileManager {
    fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir();
        let request_path = temp_dir.join(format!("llm_request_{}.json", random::<u64>()));
        let response_path = temp_dir.join(format!("llm_response_{}.json", random::<u64>()));
        
        Ok(Self {
            request_path,
            response_path,
        })
    }
    
    fn write_request<T: serde::Serialize>(&self, request: &T) -> Result<()> {
        let json = serde_json::to_string(request)?;
        fs::write(&self.request_path, json)?;
        Ok(())
    }
    
    fn read_response(&self) -> Result<String> {
        fs::read_to_string(&self.response_path)
            .map_err(|e| anyhow!("Failed to read response file: {}", e))
    }
    
    fn cleanup(&self) -> Result<()> {
        if self.request_path.exists() {
            fs::remove_file(&self.request_path)
                .map_err(|e| anyhow!("Failed to remove request file: {}", e))?;
        }
        
        if self.response_path.exists() {
            fs::remove_file(&self.response_path)
                .map_err(|e| anyhow!("Failed to remove response file: {}", e))?;
        }
        
        Ok(())
    }
}

impl Drop for TempFileManager {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

// LLM Client implementation
pub struct LlmClient {
    pub config: LlmRequestConfig,
}

impl LlmClient {
    pub fn new(config: LlmRequestConfig) -> Self {
        Self {
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
    pub fn send_prompt(&self, prompt: &str) -> Result<String> {
        match self.config.provider {
            LlmProvider::OpenAI => self.send_openai_prompt(prompt),
            LlmProvider::Anthropic => self.send_anthropic_prompt(prompt),
            LlmProvider::Custom(ref endpoint) => self.send_custom_prompt(endpoint, prompt),
        }
    }
    
    // Send a prompt to OpenAI using curl
    fn send_openai_prompt(&self, prompt: &str) -> Result<String> {
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
        
        // Create temporary files for request and response
        let temp_files = TempFileManager::new()?;
        temp_files.write_request(&request)?;
        
        // Build curl command
        let status = Command::new("curl")
            .arg("-s")
            .arg("-X").arg("POST")
            .arg("-H").arg("Content-Type: application/json")
            .arg("-H").arg(format!("Authorization: Bearer {}", self.config.api_key))
            .arg("-d").arg(format!("@{}", temp_files.request_path.display()))
            .arg("-o").arg(format!("{}", temp_files.response_path.display()))
            .arg("--max-time").arg(self.config.timeout_seconds.to_string())
            .arg(endpoint)
            .status()
            .map_err(|e| anyhow!("Failed to execute curl: {}", e))?;
            
        if !status.success() {
            return Err(anyhow!("Curl command failed with status: {}", status));
        }
        
        // Read and parse the response
        let response_text = temp_files.read_response()?;
        let response_data: Result<OpenAIResponse, _> = serde_json::from_str(&response_text);
        
        match response_data {
            Ok(data) => {
                if data.choices.is_empty() {
                    return Err(anyhow!("OpenAI returned no choices"));
                }
                Ok(data.choices[0].message.content.clone())
            },
            Err(e) => {
                // Check if the response contains an error message
                if let Ok(error_json) = serde_json::from_str::<Value>(&response_text) {
                    if let Some(error) = error_json.get("error") {
                        if let Some(message) = error.get("message").and_then(|m| m.as_str()) {
                            return Err(anyhow!("OpenAI API error: {}", message));
                        }
                    }
                }
                Err(anyhow!("Failed to parse OpenAI response: {}", e))
            }
        }
    }
    
    // Send a prompt to Anthropic using curl
    fn send_anthropic_prompt(&self, prompt: &str) -> Result<String> {
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
        
        // Create temporary files for request and response
        let temp_files = TempFileManager::new()?;
        temp_files.write_request(&request)?;
        
        // Build curl command
        let status = Command::new("curl")
            .arg("-s")
            .arg("-X").arg("POST")
            .arg("-H").arg("Content-Type: application/json")
            .arg("-H").arg(format!("X-API-Key: {}", self.config.api_key))
            .arg("-H").arg("Anthropic-Version: 2023-06-01")
            .arg("-d").arg(format!("@{}", temp_files.request_path.display()))
            .arg("-o").arg(format!("{}", temp_files.response_path.display()))
            .arg("--max-time").arg(self.config.timeout_seconds.to_string())
            .arg(endpoint)
            .status()
            .map_err(|e| anyhow!("Failed to execute curl: {}", e))?;
            
        if !status.success() {
            return Err(anyhow!("Curl command failed with status: {}", status));
        }
        
        // Read and parse the response
        let response_text = temp_files.read_response()?;
        let response_data: Result<AnthropicResponse, _> = serde_json::from_str(&response_text);
        
        match response_data {
            Ok(data) => Ok(data.completion),
            Err(e) => {
                // Check if the response contains an error message
                if let Ok(error_json) = serde_json::from_str::<Value>(&response_text) {
                    if let Some(error) = error_json.get("error") {
                        if let Some(message) = error.get("message").and_then(|m| m.as_str()) {
                            return Err(anyhow!("Anthropic API error: {}", message));
                        }
                    }
                }
                Err(anyhow!("Failed to parse Anthropic response: {}", e))
            }
        }
    }
    
    // Send a prompt to a custom endpoint using curl
    fn send_custom_prompt(&self, endpoint: &str, prompt: &str) -> Result<String> {
        // Simple implementation for custom endpoints
        let request = serde_json::json!({
            "prompt": prompt,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });
        
        // Create temporary files for request and response
        let temp_files = TempFileManager::new()?;
        temp_files.write_request(&request)?;
        
        // Build curl command with authorization if API key is provided
        let mut curl_cmd = Command::new("curl");
        curl_cmd.arg("-s")
            .arg("-X").arg("POST")
            .arg("-H").arg("Content-Type: application/json");
            
        if !self.config.api_key.is_empty() {
            curl_cmd.arg("-H").arg(format!("Authorization: Bearer {}", self.config.api_key));
        }
        
        let status = curl_cmd
            .arg("-d").arg(format!("@{}", temp_files.request_path.display()))
            .arg("-o").arg(format!("{}", temp_files.response_path.display()))
            .arg("--max-time").arg(self.config.timeout_seconds.to_string())
            .arg(endpoint)
            .status()
            .map_err(|e| anyhow!("Failed to execute curl: {}", e))?;
            
        if !status.success() {
            return Err(anyhow!("Curl command failed with status: {}", status));
        }
        
        // Read the response
        let response_text = temp_files.read_response()?;
        
        // Try to parse as JSON and extract common response fields
        if let Ok(response_json) = serde_json::from_str::<Value>(&response_text) {
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
            } else if let Some(choices) = response_json.get("choices").and_then(|v| v.as_array()) {
                if !choices.is_empty() {
                    if let Some(message) = choices[0].get("message") {
                        if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                            return Ok(content.to_string());
                        }
                    } else if let Some(text) = choices[0].get("text").and_then(|v| v.as_str()) {
                        return Ok(text.to_string());
                    }
                }
            }
            
            // If we couldn't extract a specific field, return the whole JSON as a string
            return Ok(serde_json::to_string_pretty(&response_json)
                .unwrap_or_else(|_| "Failed to format response".to_string()));
        }
        
        // If not JSON, return the raw text
        Ok(response_text)
    }
}
