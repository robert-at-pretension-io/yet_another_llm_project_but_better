use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::Result;
use tempfile;
use thiserror::Error;

use crate::parser::{Block, parse_document, extract_variable_references};
use crate::llm_client::{LlmClient, LlmRequestConfig, LlmProvider};

// mod enhanced_variables;  // Module was deleted
mod reference_resolver;
use reference_resolver::ReferenceResolver;

// Define error type
#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Circular dependency: {0}")]
    CircularDependency(String),
    
    #[error("Missing fallback: {0}")]
    MissingFallback(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("LLM API error: {0}")]
    LlmApiError(String),
    
    #[error("Missing API key: {0}")]
    MissingApiKey(String),
    
    #[error("Failed to resolve reference: {0}")]
    ReferenceResolutionFailed(String)
}

// Executor for processing blocks
pub struct MetaLanguageExecutor {
    // Store named blocks and their outputs
    pub blocks: HashMap<String, Block>,
    pub outputs: HashMap<String, String>,
    pub fallbacks: HashMap<String, String>,
    // Cache results for blocks with cache_result:true
    pub cache: HashMap<String, (String, Instant)>,
    // Execution context
    pub current_document: String,
    // Track blocks being processed to detect circular dependencies
    processing_blocks: Vec<String>,
    // Track if this is a new or existing executor
    pub instance_id: String
}

impl MetaLanguageExecutor {

    pub fn new() -> Self {
        // Generate a unique ID for this executor instance
        let instance_id = format!("executor_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());
        
        println!("DEBUG: Creating new executor instance: {}", instance_id);
        
        Self {
            blocks: HashMap::new(),
            outputs: HashMap::new(),
            fallbacks: HashMap::new(),
            cache: HashMap::new(),
            current_document: String::new(),
            processing_blocks: Vec::new(),
            instance_id,
        }
    }
    
    // Debug method to print the current state of outputs
    pub fn debug_print_outputs(&self, context: &str) {
        println!("\nDEBUG: [{}] Executor {} outputs state:", context, self.instance_id);
        println!("DEBUG: Total outputs: {}", self.outputs.len());
        
        if self.outputs.is_empty() {
            println!("DEBUG: No outputs stored.");
        } else {
            for (key, value) in &self.outputs {
                let preview = if value.len() > 50 {
                    format!("{}... (length: {})", &value[..50], value.len())
                } else {
                    format!("{} (length: {})", value, value.len())
                };
                println!("DEBUG:   '{}' => '{}'", key, preview);
            }
        }
        println!("DEBUG: End of outputs state\n");
    }

    // Process a document
    pub fn process_document(&mut self, content: &str) -> Result<(), ExecutorError> {
        println!("DEBUG: Processing document with executor: {}", self.instance_id);
        
        // Set environment variable to preserve variable references in original block content
        std::env::set_var("LLM_PRESERVE_REFS", "1");
        
        // Debug: Print the current state of outputs before processing
        self.debug_print_outputs("BEFORE PROCESSING");
        
        // Parse the document
        let blocks = parse_document(content).map_err(|e| ExecutorError::ExecutionFailed(e.to_string()))?;
        println!("DEBUG: Parsed {} blocks from document", blocks.len());
        
        // Debug: Print summary of parsed blocks
        for (i, block) in blocks.iter().enumerate() {
            println!("DEBUG: Block {}: type='{}', name={:?}, content_length={}", 
                     i, block.block_type, block.name, block.content.len());
        }
        
        // Store the current outputs before clearing
        let previous_outputs = self.outputs.clone();
        println!("DEBUG: Preserved {} previous outputs before clearing", previous_outputs.len());
        
        // Clear existing state (keeping cache)
        self.blocks.clear();
        self.outputs.clear();
        self.fallbacks.clear();
        self.current_document = content.to_string();
        self.processing_blocks.clear();
        
        // Register all blocks and identify fallbacks
        for (index, block) in blocks.iter().enumerate() {
            // Generate a name for the block if it doesn't have one
            let block_key = if let Some(name) = &block.name {
                name.clone()
            } else {
                // Generate a unique name based on block type and index
                let generated_name = format!("{}_{}", block.block_type, index);
                println!("DEBUG: Generated name '{}' for unnamed block of type '{}'", 
                         generated_name, block.block_type);
                generated_name
            };
            
            println!("DEBUG: Registering block '{}' of type '{}' in executor", 
                     block_key, block.block_type);
            self.blocks.insert(block_key.clone(), block.clone());
            
            // Check if this is a fallback block
            if let Some(name) = &block.name {
                if name.ends_with("-fallback") {
                    let original_name = name.trim_end_matches("-fallback");
                    self.fallbacks.insert(original_name.to_string(), name.clone());
                    println!("DEBUG: Registered fallback '{}' for block '{}'", 
                             name, original_name);
                }
                
                // Store content of data blocks directly in outputs
                if block.block_type == "data" {
                    self.outputs.insert(name.clone(), block.content.clone());
                    println!("DEBUG: Stored data block '{}' in outputs", name);
                }
            }
        }
        
        // Restore previous responses that aren't in the current document
        // This preserves LLM responses between document edits
        let mut restored_count = 0;
        for (key, value) in previous_outputs {
            // Only restore responses, not other outputs
            if key.ends_with("_response") || key == "question_response" {
                // Check if this response isn't already in the outputs map
                if !self.outputs.contains_key(&key) {
                    self.outputs.insert(key.clone(), value);
                    restored_count += 1;
                    println!("DEBUG: Restored previous response: '{}'", key);
                }
            }
        }
        println!("DEBUG: Restored {} previous responses", restored_count);
        
        // Process variable references in all registered blocks using a two-phase approach
        let mut ops = Vec::new();
        for (name, _) in self.blocks.iter() {
            ops.push(name.clone());
        }
        
        // Now process each block's content and update both the block and outputs
        for name in ops {
            if let Some(block) = self.blocks.get(&name) {
                let content = block.content.clone();
                let processed_content = self.process_variable_references(&content);
                let is_executable = self.is_executable_block(block);
                
                // Only update if content changed
                if processed_content != content {
                    if let Some(block_mut) = self.blocks.get_mut(&name) {
                        block_mut.content = processed_content.clone();
                    }
                    
                    // Also update outputs for non-executable blocks
                    if !is_executable {
                        let modified_content = self.apply_block_modifiers_to_variable(&name, &processed_content);
                        self.outputs.insert(name.clone(), modified_content);
                    }
                }
            }
        }
        
        // Register fallbacks for executable blocks that don't have them
        for block in &blocks {
            if let Some(name) = &block.name {
                if self.is_executable_block(&block) && !self.has_fallback(name) {
                    println!("Warning: Executable block '{}' has no fallback defined", name);
                    // In a real implementation, would generate a default fallback
                }
            }
        }
        
        // Now process executable blocks that don't depend on other blocks
        for block in blocks {
            // Use the block's name if available, otherwise generate one
            let block_key = if let Some(name) = &block.name {
                name.clone()
            } else {
                // Look up the generated name in the blocks map
                let block_type = &block.block_type;
                self.blocks.iter()
                    .find(|(_, b)| b.block_type == *block_type && b.content == block.content)
                    .map(|(k, _)| k.clone())
                    .unwrap_or_else(|| format!("{}_unknown", block_type))
            };
            
            if self.is_executable_block(&block) && !self.has_explicit_dependency(&block) {
                println!("DEBUG: Executing independent block: '{}'", block_key);
                self.execute_block(&block_key)?;
            } else {
                println!("DEBUG: Skipping non-executable or dependent block: '{}'", block_key);
            }
        }
        
        Ok(())
    }
    
    /// Helper function to process <meta:reference> tags using XML parsing
    // Check if a block is executable
    pub fn is_executable_block(&self, block: &Block) -> bool {
        matches!(block.block_type.as_str(), 
                "code:python" | "code:javascript" | "code:rust" | 
                "shell" | "api" | "question")
    }
    
    // Check if a block is a data block
    pub fn is_data_block(&self, block: &Block) -> bool {
        block.block_type == "data" || block.block_type.starts_with("data:")
    }
    
    // Check if a block has a fallback defined
    pub fn has_fallback(&self, name: &str) -> bool {
        self.fallbacks.contains_key(name)
    }
    
    // Check if a block has explicit dependencies
    pub fn has_explicit_dependency(&self, block: &Block) -> bool {
        block.modifiers.iter().any(|(key, _)| key == "depends" || key == "requires")
    }
    
    // Execute a block by name
    pub fn execute_block(&mut self, name: &str) -> Result<String, ExecutorError> {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Executing block: '{}'", name);
        }
        
        // Check for circular dependencies
        if self.processing_blocks.contains(&name.to_string()) {
            println!("ERROR: Circular dependency detected for block: '{}'", name);
            return Err(ExecutorError::CircularDependency(name.to_string()));
        }
        
        // Check if block exists
        let block = match self.blocks.get(name) {
            Some(b) => b.clone(),
            None => {
                println!("ERROR: Block not found: '{}'", name);
                return Err(ExecutorError::BlockNotFound(name.to_string()));
            }
        };
        
        // Check if result is cached
        if self.is_cacheable(&block) {
            if let Some((result, timestamp)) = self.cache.get(name) {
                // Check if cache is still valid (e.g., within timeout)
                let now = Instant::now();
                let timeout = self.get_timeout(&block);
                let elapsed = now.duration_since(*timestamp);
                
                let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
                
                if elapsed < timeout {
                    if debug_enabled {
                        println!("DEBUG: Using cached result for '{}' (age: {:.2}s, timeout: {}s)", 
                                 name, elapsed.as_secs_f64(), timeout.as_secs());
                    }
                    return Ok(result.clone());
                } else if debug_enabled {
                    println!("DEBUG: Cache expired for '{}' (age: {:.2}s, timeout: {}s)", 
                             name, elapsed.as_secs_f64(), timeout.as_secs());
                }
            }
        }
        
        // Mark block as being processed (for dependency tracking)
        self.processing_blocks.push(name.to_string());
        
        // Execute dependencies first
        for (key, value) in &block.modifiers {
            if key == "depends" || key == "requires" {
                let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
                
                if debug_enabled {
                    println!("DEBUG: Block '{}' depends on '{}', executing dependency first", name, value);
                }
                
                self.execute_block(value)?;
                
                if debug_enabled {
                    println!("DEBUG: Dependency '{}' executed successfully, continuing with '{}'", value, name);
                }
            }
        }
        
        // Process variable references in content
        // We need to get the latest content from the blocks map, as it might have been updated
        let block_content = if let Some(updated_block) = self.blocks.get(name) {
            updated_block.content.clone()
        } else {
            block.content.clone()
        };
    
        // Process variable references and conditional blocks
        let processed_content =block_content;      
        // Execute based on block type
        let result = match block.block_type.as_str() {
            "shell" => self.execute_shell(&processed_content),
            "api" => self.execute_api(&processed_content),
            "question" => self.execute_question(&block, &processed_content),
            _ => {
                // Default to returning the processed content
                Ok(processed_content)
            }
        };
        
        // Remove block from processing list
        self.processing_blocks.retain(|b| b != name);
        
        // Handle execution result
        match result {
            Ok(output) => {
                let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
                
                // Store output with the block name
                self.outputs.insert(name.to_string(), output.clone());
                
                // Also store with block_name.results format
                let results_key = format!("{}.results", name);
                self.outputs.insert(results_key, output.clone());
                
                // Also store with block_name_results format for compatibility
                let results_key = format!("{}_results", name);
                self.outputs.insert(results_key, output.clone());
                
                if let Some(b) = self.blocks.get_mut(name) {
                    b.content = output.clone();
                }
                
                if debug_enabled {
                    println!("DEBUG: Block '{}' executed successfully, output length: {}", 
                             name, output.len());
                }
                
                // Cache if needed
                if self.is_cacheable(&block) {
                    if debug_enabled {
                        println!("DEBUG: Caching result for block '{}'", name);
                    }
                    self.cache.insert(name.to_string(), (output.clone(), Instant::now()));
                }
                
                Ok(output)
            },
            Err(e) => {
                let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
                
                // Store error with block_name_error format
                let error_key = format!("{}_error", name);
                self.outputs.insert(error_key, e.to_string());
                
                // Create an error-response block
                if let Some(block) = self.blocks.get(name) {
                    let error_response_name = format!("{}_error_response", name);
                    let error_str = e.to_string();
                    let error_response_block = self.generate_error_response_block(block, &error_str);
                    
                    // Store the error-response block
                    self.blocks.insert(error_response_name.clone(), error_response_block);
                    
                    // Store the error response in outputs
                    self.outputs.insert(error_response_name, error_str);
                    
                    if debug_enabled {
                        println!("DEBUG: Created error-response block for '{}'", name);
                    }
                }
                
                // Use fallback
                if let Some(fallback_name) = self.fallbacks.get(name) {
                    if debug_enabled {
                        println!("DEBUG: Block '{}' failed with error: {}", name, e);
                        println!("DEBUG: Using fallback: {}", fallback_name);
                    } else {
                        println!("Block '{}' failed, using fallback: {}", name, fallback_name);
                    }
                    
                    let fallback_name_clone = fallback_name.clone();
                    self.execute_block(&fallback_name_clone)
                } else {
                    if debug_enabled {
                        println!("DEBUG: Block '{}' failed with error: {}", name, e);
                        println!("DEBUG: No fallback available");
                    }
                    Err(e)
                }
            }
        }
    }
    
    // Check if a block's result should be cached
    pub fn is_cacheable(&self, block: &Block) -> bool {
        // First check if caching is globally disabled via environment variable
        if let Ok(cache_disabled) = std::env::var("LLM_NO_CACHE") {
            if cache_disabled == "1" || cache_disabled.to_lowercase() == "true" {
                return false;
            }
        }
        
        // Then check block modifiers
        block.modifiers.iter().any(|(key, value)| 
            key == "cache_result" && 
            (value == "true" || value == "yes" || value == "1" || value == "on")
        )
    }
    
    // Get timeout duration for a block
    pub fn get_timeout(&self, block: &Block) -> Duration {
        // First check block modifiers
        for (key, value) in &block.modifiers {
            if key == "timeout" {
                if let Ok(seconds) = value.parse::<u64>() {
                    println!("DEBUG: Using block timeout: {} seconds", seconds);
                    return Duration::from_secs(seconds);
                }
            }
        }
        
        // Then check environment variable
        if let Ok(timeout_str) = std::env::var("LLM_TIMEOUT") {
            if let Ok(seconds) = timeout_str.parse::<u64>() {
                println!("DEBUG: Using environment timeout: {} seconds", seconds);
                return Duration::from_secs(seconds);
            }
        }
        
        // Default timeout (10 minutes)
        println!("DEBUG: Using default timeout: 600 seconds");
        Duration::from_secs(600)
    }
    
    pub fn determine_format_from_content(&self, content: &str) -> &'static str {
        // Trim whitespace
        let trimmed = content.trim();
        
        // Check if it looks like JSON (object or array)
        if (trimmed.starts_with('{') && trimmed.ends_with('}')) || 
           (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            // Try to parse as JSON to validate
            if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
                return "json";
            }
        }
        
        // Check for other formats (could be expanded)
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            return "xml";
        }
        
        // Default to plain text
        "text"
    }
    
    pub fn execute_shell(&self, command: &str) -> Result<String, ExecutorError> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", command])
                .output()?
        } else {
            Command::new("sh")
                .args(&["-c", command])
                .output()?
        };
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }
    
    pub fn execute_api(&self, url: &str) -> Result<String, ExecutorError> {
        // In a real implementation, this would use a proper HTTP client
        // and handle different HTTP methods, headers, etc.
        let output = Command::new("curl")
            .arg("-s")
            .arg(url)
            .output()?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }
    
    // Execute a question block by sending it to an LLM API
    pub fn execute_question(&mut self, block: &Block, question: &str) -> Result<String, ExecutorError> {
        println!("DEBUG: Executing question block: {}", question);
        println!("DEBUG: Block name: {:?}", block.name);
        println!("DEBUG: Block modifiers: {:?}", block.modifiers);
        
        // Check if we're in test mode - more robust environment variable checking
        let test_mode_env = std::env::var("LLM_TEST_MODE").unwrap_or_default();
        let is_test_mode = block.is_modifier_true("test_mode") || 
                          !test_mode_env.is_empty() || 
                          test_mode_env == "1" || 
                          test_mode_env.to_lowercase() == "true";
        
        if is_test_mode {
            println!("DEBUG: Test mode detected (env: '{}', block modifier: {})", 
                     test_mode_env, block.is_modifier_true("test_mode"));
            
            let test_response = if let Some(test_response) = block.get_modifier("test_response") {
                println!("DEBUG: Using custom test response from modifier");
                test_response.clone()
            } else {
                println!("DEBUG: Using default test mode response");
                "This is a simulated response for testing purposes.".to_string()
            };
            return Ok(test_response);
        }
        
        // Create LLM client from block modifiers
        let llm_client = LlmClient::from_block_modifiers(&block.modifiers);
        println!("DEBUG: Created LLM client with provider: {:?}", llm_client.config.provider);
        
        // Check if we have an API key
        if llm_client.config.api_key.is_empty() {
            println!("DEBUG: Missing API key");
            return Err(ExecutorError::MissingApiKey(
                "No API key provided for LLM. Set via block modifier or environment variable.".to_string()
            ));
        }
        
        // Prepare the prompt
        let mut prompt = question.to_string();
        println!("DEBUG: Initial prompt: {}", prompt);
        
        // Check if there's a system prompt modifier
        if let Some(system_prompt) = block.get_modifier("system_prompt") {
            // For OpenAI, we'd format this differently, but for simplicity we'll just prepend
            prompt = format!("{}\n\n{}", system_prompt, prompt);
            println!("DEBUG: Added system prompt, new prompt length: {}", prompt.len());
        }
        
        // Check if there's a context modifier that references other blocks
        if let Some(context_block) = block.get_modifier("context") {
            println!("DEBUG: Found context block reference: {}", context_block);
            if let Some(context_content) = self.outputs.get(context_block) {
                println!("DEBUG: Found context content, length: {}", context_content.len());
                prompt = format!("Context:\n{}\n\nQuestion:\n{}", context_content, prompt);
            } else {
                println!("DEBUG: Context block '{}' not found in outputs", context_block);
            }
        }
        // Get any additional context from direct context modifier
        else if let Some(context) = block.get_modifier("context") {
            println!("DEBUG: Using direct context, length: {}", context.len());
            prompt = format!("Context:\n{}\n\nQuestion:\n{}", context, prompt);
        }
        
        println!("DEBUG: Final prompt length: {}", prompt.len());
        
        // Execute the LLM request using the synchronous client
        println!("DEBUG: Sending prompt to LLM API");
        let result = match llm_client.send_prompt(&prompt) {
            Ok(response) => {
                println!("DEBUG: Received successful response from LLM, length: {}", response.len());
                Ok(response)
            },
            Err(e) => {
                println!("DEBUG: LLM API error: {}", e);
                Err(ExecutorError::LlmApiError(e.to_string()))
            },
        };
        
        // Process the result
        match result {
            Ok(response) => {
                println!("DEBUG: Processing successful response");
                
                // Create a response block
                if let Some(name) = &block.name {
                    // For named question blocks
                    let response_block_name = format!("{}_response", name);
                    println!("DEBUG: Creating response block: {}", response_block_name);
                    
                    let response_str = response.as_str();
                    let mut response_block = Block::new("response", Some(&response_block_name), response_str);
                    println!("DEBUG: Created response block with content length: {}", response_str.len());
                    println!("DEBUG: Storing response in executor: {}", self.instance_id);
                    
                    // Copy relevant modifiers from the question block
                    for (key, value) in &block.modifiers {
                        if matches!(key.as_str(), "format" | "display" | "max_lines" | "trim") {
                            println!("DEBUG: Copying modifier from question to response: {}={}", key, value);
                            response_block.add_modifier(key, value);
                        }
                    }
                    
                    // Add reference back to the question block
                    response_block.add_modifier("for", name);
                    println!("DEBUG: Added 'for' modifier pointing to: {}", name);
                    
                    // Store the response block
                    println!("DEBUG: Storing response block in blocks map");
                    self.blocks.insert(response_block_name.clone(), response_block);
                    
                    // Store the response in outputs
                    println!("DEBUG: Storing response in outputs map with key: {}", response_block_name);
                    self.outputs.insert(response_block_name, response.clone());
                } else {
                    // For unnamed question blocks
                    println!("DEBUG: Question block has no name, creating generic response block");
                    
                    let response_str = response.as_str();
                    let mut response_block = Block::new("response", Some("generic_response"), response_str);
                    println!("DEBUG: Created generic response block with content length: {}", response_str.len());
                    
                    // Copy relevant modifiers from the question block
                    for (key, value) in &block.modifiers {
                        if matches!(key.as_str(), "format" | "display" | "max_lines" | "trim") {
                            println!("DEBUG: Copying modifier from question to response: {}={}", key, value);
                            response_block.add_modifier(key, value);
                        }
                    }
                    
                    // Store the response block
                    println!("DEBUG: Storing generic response block in blocks map");
                    self.blocks.insert("generic_response".to_string(), response_block);
                    
                    // Store the response in outputs with a generic key
                    println!("DEBUG: Storing response in outputs map with key: question_response");
                    self.outputs.insert("question_response".to_string(), response.clone());
                }
                
                // Debug: Print all outputs after adding this one
                println!("DEBUG: Current outputs after adding response:");
                for (k, v) in &self.outputs {
                    println!("DEBUG:   '{}' => '{}' (length: {})", k, 
                             if v.len() > 30 { &v[..30] } else { v }, v.len());
                }
                
                Ok(response)
            },
            Err(e) => {
                println!("DEBUG: Processing error response for question: {}", e);
                
                // Create an error-response block
                if let Some(name) = &block.name {
                    // For named question blocks
                    let error_response_name = format!("{}_error_response", name);
                    println!("DEBUG: Creating error-response block: {}", error_response_name);
                    
                    let error_str = e.to_string();
                    let error_response_block = self.generate_error_response_block(block, &error_str);
                    
                    // Store the error-response block
                    println!("DEBUG: Storing error-response block in blocks map");
                    self.blocks.insert(error_response_name.clone(), error_response_block);
                    
                    // Store the error response in outputs
                    println!("DEBUG: Storing error response in outputs map with key: {}", error_response_name);
                    self.outputs.insert(error_response_name, error_str.clone());
                    
                    // Also store with the standard error key format for compatibility
                    let error_key = format!("{}_error", name);
                    self.outputs.insert(error_key, error_str);
                } else {
                    // For unnamed question blocks
                    println!("DEBUG: Question block has no name, creating generic error-response block");
                    
                    let error_str = e.to_string();
                    let error_response_block = self.generate_error_response_block(block, &error_str);
                    
                    // Store the error-response block
                    println!("DEBUG: Storing generic error-response block in blocks map");
                    self.blocks.insert("generic_error_response".to_string(), error_response_block);
                    
                    // Store the error response in outputs with a generic key
                    println!("DEBUG: Storing error response in outputs map with key: question_error_response");
                    self.outputs.insert("question_error_response".to_string(), error_str);
                }
                
                println!("DEBUG: Returning error from execute_question: {}", e);
                Err(e)
            },
        }
    }
    
    // Generate a results block for an executed block
    pub fn generate_results_block(&self, block: &Block, output: &str, format: Option<String>) -> Block {
        let mut results_block = Block::new("results", None, output);
        
        // Add "for" modifier pointing to the original block
        if let Some(block_name) = &block.name {
            results_block.add_modifier("for", block_name);
        }
        
        // Apply default display setting
        results_block.add_modifier("display", "block");
        
        // Use specified format or inherit from the original block
        if let Some(format_val) = format {
            results_block.add_modifier("format", &format_val);
        } else if let Some(display) = block.get_modifier("format") {
            results_block.add_modifier("format", display);
        }
        
        // Inherit other relevant modifiers from the original block
        if let Some(display) = block.get_modifier("display") {
            results_block.add_modifier("display", display);
        }
        
        if let Some(max_lines) = block.get_modifier("max_lines") {
            results_block.add_modifier("max_lines", max_lines);
        }
        
        if let Some(trim_value) = block.get_modifier("trim") {
            results_block.add_modifier("trim", trim_value);
        }
        
        results_block
    }
    
    // Generate an error results block for a failed execution
    pub fn generate_error_results_block(&self, block: &Block, error: &str) -> Block {
        let mut error_block = Block::new("error_results", None, error);
        
        // Add "for" modifier pointing to the original block
        if let Some(block_name) = &block.name {
            error_block.add_modifier("for", block_name);
        }
        
        error_block
    }
    
    // Generate an error-response block from a question or code block
    pub fn generate_error_response_block(&self, original_block: &Block, error_text: &str) -> Block {
        println!("DEBUG: generate_error_response_block called");
        println!("DEBUG: Original block name: {:?}", original_block.name);
        println!("DEBUG: Error text length: {}", error_text.len());
        
        let response_name = if let Some(name) = &original_block.name {
            let name = format!("{}_error_response", name);
            println!("DEBUG: Generated error response name: {}", name);
            Some(name)
        } else {
            println!("DEBUG: No name for original block, error response will be unnamed");
            None
        };
        
        let mut error_response_block = Block::new("error-response", response_name.as_deref(), error_text);
        println!("DEBUG: Created error-response block with type: {}", error_response_block.block_type);
        
        // Add "for" modifier pointing to the original block
        if let Some(block_name) = &original_block.name {
            println!("DEBUG: Adding 'for' modifier with value: {}", block_name);
            error_response_block.add_modifier("for", block_name);
        }
        
        // Copy relevant modifiers from the original block
        for (key, value) in &original_block.modifiers {
            if matches!(key.as_str(), "format" | "display" | "max_lines" | "trim") {
                println!("DEBUG: Copying modifier: {}={}", key, value);
                error_response_block.add_modifier(key, value);
            }
        }
        
        // Set default format to markdown if not specified
        if !original_block.modifiers.iter().any(|(k, _)| k == "format") {
            println!("DEBUG: Setting default format to markdown");
            error_response_block.add_modifier("format", "markdown");
        }
        
        println!("DEBUG: Final error-response block modifiers: {:?}", error_response_block.modifiers);
        error_response_block
    }

    // Generate a response block from a question block
    pub fn generate_response_block(&self, question_block: &Block, response_text: &str) -> Block {
        println!("DEBUG: generate_response_block called");
        println!("DEBUG: Question block name: {:?}", question_block.name);
        println!("DEBUG: Response text length: {}", response_text.len());
        
        let response_name = if let Some(name) = &question_block.name {
            let name = format!("{}_response", name);
            println!("DEBUG: Generated response name: {}", name);
            Some(name)
        } else {
            println!("DEBUG: No name for question block, response will be unnamed");
            None
        };
        
        let mut response_block = Block::new("response", response_name.as_deref(), response_text);
        println!("DEBUG: Created response block with type: {}", response_block.block_type);
        
        // Add "for" modifier pointing to the original question block
        if let Some(block_name) = &question_block.name {
            println!("DEBUG: Adding 'for' modifier with value: {}", block_name);
            response_block.add_modifier("for", block_name);
        }
        
        // Copy relevant modifiers from the question block
        for (key, value) in &question_block.modifiers {
            if matches!(key.as_str(), "format" | "display" | "max_lines" | "trim") {
                println!("DEBUG: Copying modifier: {}={}", key, value);
                response_block.add_modifier(key, value);
            }
        }
        
        // Set default format to markdown if not specified
        if !question_block.modifiers.iter().any(|(k, _)| k == "format") {
            println!("DEBUG: Setting default format to markdown");
            response_block.add_modifier("format", "markdown");
        }
        
        println!("DEBUG: Final response block modifiers: {:?}", response_block.modifiers);
        response_block
    }
    
    // Apply trim modifier to results content
    pub fn apply_trim(&self, block: &Block, content: &str) -> String {
        if let Some(trim_value) = block.get_modifier("trim") {
            match trim_value.as_str() {
                "true" | "yes" | "1" => return content.trim().to_string(),
                "start" | "left" => return content.trim_start().to_string(),
                "end" | "right" => return content.trim_end().to_string(),
                "lines" => {
                    // Trim each line individually
                    return content.lines()
                        .map(|line| line.trim())
                        .collect::<Vec<&str>>()
                        .join("\n");
                }
                _ => {}
            }
        }
        
        content.to_string()
    }
    
    // Apply max_lines modifier to results content
    pub fn apply_max_lines(&self, block: &Block, content: &str) -> String {
        if let Some(max_lines_str) = block.get_modifier("max_lines") {
            if let Ok(max_lines) = max_lines_str.parse::<usize>() {
                if max_lines > 0 {
                    let lines: Vec<&str> = content.lines().collect();
                    if lines.len() > max_lines {
                        let mut result = lines[..max_lines].join("\n");
                        
                        // Add ellipsis indicator if truncated
                        if let Some(ellipsis) = block.get_modifier("ellipsis") {
                            result.push_str(&format!("\n{}", ellipsis));
                        } else {
                            result.push_str("\n...");
                        }
                        
                        return result;
                    }
                }
            }
        }
        
        content.to_string()
    }
    
    // Process results content with all applicable modifiers
    pub fn process_results_content(&self, block: &Block, content: &str) -> String {
        // Apply modifiers in sequence
        let trimmed = self.apply_trim(block, content);
        let truncated = self.apply_max_lines(block, &trimmed);
        
        // Apply additional formatting based on modifiers
        if let Some(format_type) = block.get_modifier("format") {
            match format_type.as_str() {
                "json" => {
                    // Try to pretty-print JSON if requested
                    if block.has_modifier("pretty") {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&truncated) {
                            if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                                return pretty;
                            }
                        }
                    }
                }
                // Add other format-specific processing as needed
                _ => {}
            }
        }
        
        truncated
    }
    
    
    // Update document with execution results
    pub fn update_document(&self) -> Result<String, ExecutorError> {
        // This is a simplified version - a real implementation would be more sophisticated
        // to properly handle block updates without losing formatting
        
        println!("DEBUG: update_document called");
        println!("DEBUG: Current document length: {}", self.current_document.len());
        println!("DEBUG: Number of outputs: {}", self.outputs.len());
        
        // Set environment variables to control variable reference processing
        std::env::set_var("LLM_PROCESS_REFS_IN_EXECUTOR", "1");
        std::env::remove_var("LLM_PRESERVE_REFS");
        
        // Check if the document contains any limit modifiers
        let contains_limit_modifier = self.current_document.contains("${") && 
                                     self.current_document.contains(":limit=");
        
        let mut updated_doc = self.current_document.clone();
        
        if contains_limit_modifier {
            println!("DEBUG: Document contains limit modifiers, ensuring they are processed");
            // Process the document to handle limit modifiers specifically
            updated_doc = self.process_variable_references(&self.current_document);
            println!("DEBUG: Processed document for limit modifiers, new length: {}", updated_doc.len());
        }
        
        // Debug: Print all outputs
        println!("DEBUG: All outputs:");
        for (k, v) in &self.outputs {
            println!("DEBUG:   '{}' => '{}' (length: {})", k, 
                     if v.len() > 30 { &v[..30] } else { v }, v.len());
        }
        
        // Process any reference blocks in the document
        // First, let's check if there are any reference blocks in the document
        let contains_references = updated_doc.contains("<meta:reference");
        
        if contains_references {
            println!("DEBUG: Document contains reference elements, replacing them with content");
            
            // Get all blocks that are references
            let reference_blocks: Vec<(String, &Block)> = self.blocks.iter()
                .filter(|(_, b)| b.block_type == "reference")
                .map(|(n, b)| (n.clone(), b))
                .collect();
            
            // Process each reference block
            for (name, block) in reference_blocks {
                if let Some(target) = block.get_modifier("target") {
                    // Look for a specific reference tag pattern
                    let reference_pattern = format!("<meta:reference[^>]*target=\"{}\"[^>]*/>", regex::escape(target));
                    
                    // Create a regex to find this pattern
                    if let Ok(re) = regex::Regex::new(&reference_pattern) {
                        // Get the content to replace with
                        if let Some(content) = self.outputs.get(&name) {
                            println!("DEBUG: Replacing reference to '{}' with content (length: {})", target, content.len());
                            
                            // Replace the reference tag with the content
                            updated_doc = re.replace_all(&updated_doc, content).to_string();
                        }
                    }
                }
            }
        }
        
        // Debug: Print the current state of outputs after processing
        self.debug_print_outputs("AFTER PROCESSING");
        if updated_doc.contains("<meta:") {
            println!("DEBUG: Detected XML document, updating <meta:results> blocks.");
            for (name, output) in &self.outputs {
                let double_quote_pattern = format!(r#"(?s)(<meta:code[^>]*name\s*=\s*"{}"[^>]*>.*?</meta:code>)"#, regex::escape(name));
                let mut found_match = false;
                if let Ok(re_double) = regex::Regex::new(&double_quote_pattern) {
                    let new_doc = re_double.replace_all(&updated_doc, |caps: &regex::Captures| {
                        let code_block = caps.get(1).unwrap().as_str();
                        let re_results = regex::Regex::new(&format!(r#"(?s)(<meta:results\s+name=["']{}_results["']\s+for=["']{}["'][^>]*><!\[CDATA\[).*?(]]></meta:results>)"#, regex::escape(name), regex::escape(name))).unwrap();
                        if re_results.is_match(code_block) {
                            re_results.replace_all(code_block, |caps: &regex::Captures| {
                                format!("{}{}{}", &caps[1], output, &caps[2])
                            }).to_string()
                        } else {
                            format!("{}<meta:results name='{}_results' for='{}'><![CDATA[{}]]></meta:results>", code_block, name, name, output)
                        }
                    }).to_string();
                    if new_doc != updated_doc {
                        updated_doc = new_doc;
                        found_match = true;
                    }
                } else {
                    println!("DEBUG: Regex error for double quote pattern");
                }
                if !found_match {
                    let single_quote_pattern = format!(r#"(?s)(<meta:code[^>]*name\s*=\s*'{}'[^>]*>.*?</meta:code>)"#, regex::escape(name));
                    if let Ok(re_single) = regex::Regex::new(&single_quote_pattern) {
                        updated_doc = re_single.replace_all(&updated_doc, |caps: &regex::Captures| {
                            let code_block = caps.get(1).unwrap().as_str();
                            let re_results = regex::Regex::new(&format!(r#"(?s)(<meta:results\s+name=["']{}_results["']\s+for=["']{}["'][^>]*><!\[CDATA\[).*?(]]></meta:results>)"#, regex::escape(name), regex::escape(name))).unwrap();
                            if re_results.is_match(code_block) {
                                re_results.replace_all(code_block, |caps: &regex::Captures| {
                                    format!("{}{}{}", &caps[1], output, &caps[2])
                                }).to_string()
                            } else {
                                format!("{}<meta:results name='{}_results' for='{}'><![CDATA[{}]]></meta:results>", code_block, name, name, output)
                            }
                        }).to_string();
                    } else {
                        println!("DEBUG: Regex error for single quote pattern");
                    }
                }
            }
            println!("DEBUG: Updated XML document with <meta:results> blocks. New length: {}", updated_doc.len());
            return Ok(updated_doc);
        }
        
        // Replace response blocks with execution results
        for (name, output) in &self.outputs {
            println!("DEBUG: Processing output for '{}' (length: {})", name, output.len());
            
            // Very simple replacement - in a real implementation, this would be more robust
            let response_marker = format!("[response for:{}]", name);
            let response_replacement = format!("[response for:{}]\n{}\n[/response]", name, output);
            
            println!("DEBUG: Looking for marker: '{}'", response_marker);
            let marker_count = updated_doc.matches(&response_marker).count();
            println!("DEBUG: Found {} instances of marker", marker_count);
            
            updated_doc = updated_doc.replace(&response_marker, &response_replacement);
            
            // Handle error-response markers
            let error_response_marker = format!("[error-response for:{}]", name);
            if updated_doc.contains(&error_response_marker) {
                let error_response_replacement = format!("[error-response for:{}]\n{}\n[/error-response]", name, output);
                println!("DEBUG: Looking for error-response marker: '{}'", error_response_marker);
                updated_doc = updated_doc.replace(&error_response_marker, &error_response_replacement);
            }
            
            // Also handle question-response pairs
            if name.ends_with("_response") {
                let question_name = name.trim_end_matches("_response");
                println!("DEBUG: Found response for question: '{}'", question_name);
                
                let question_response_marker = format!("[response for:{}]", question_name);
                let question_response_replacement = format!("[response for:{}]\n{}\n[/response]", question_name, output);
                
                println!("DEBUG: Looking for question marker: '{}'", question_response_marker);
                let q_marker_count = updated_doc.matches(&question_response_marker).count();
                println!("DEBUG: Found {} instances of question marker", q_marker_count);
                
                updated_doc = updated_doc.replace(&question_response_marker, &question_response_replacement);
            }
            // Handle error-response for question blocks
            else if name.ends_with("_error_response") {
                let block_name = name.trim_end_matches("_error_response");
                println!("DEBUG: Found error-response for block: '{}'", block_name);
                
                let error_response_marker = format!("[error-response for:{}]", block_name);
                let error_response_replacement = format!("[error-response for:{}]\n{}\n[/error-response]", block_name, output);
                
                println!("DEBUG: Looking for error-response marker: '{}'", error_response_marker);
                updated_doc = updated_doc.replace(&error_response_marker, &error_response_replacement);
            }
        }
        
        // Check if the document already contains response blocks
        let has_response_block = updated_doc.contains("[response]") || updated_doc.contains("[/response]") ||
                                updated_doc.contains("<meta:response") || updated_doc.contains("</meta:response>") ||
                                updated_doc.contains("[error-response]") || updated_doc.contains("[/error-response]") ||
                                updated_doc.contains("<meta:error-response") || updated_doc.contains("</meta:error-response>");
        println!("DEBUG: Document already contains response blocks: {}", has_response_block);
        println!("DEBUG: Executor {} has {} outputs available for insertion", 
                 self.instance_id, self.outputs.len());
        
        // If there are already response blocks, don't add new ones
        if has_response_block {
            println!("DEBUG: Returning document with existing response blocks");
            return Ok(updated_doc);
        }
        
        // Handle question blocks by adding response blocks after them
        println!("DEBUG: Adding response blocks after question blocks");
        let mut result = String::new();
        let mut lines = updated_doc.lines().collect::<Vec<_>>();
        println!("DEBUG: Document has {} lines", lines.len());
        
        // First pass: identify question blocks and their names
        let mut question_blocks = Vec::new();
        let mut current_question_name = None;
        let mut in_question_block = false;
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Check for question block start with name attribute - handle both XML and markdown formats
            if trimmed.starts_with("[question") || trimmed.starts_with("<meta:question") || trimmed.starts_with("<question") {
                in_question_block = true;
                
                // Try to extract name from the opening tag
                if let Some(name_start) = trimmed.find("name:") {
                    let name_start = name_start + 5; // skip "name:"
                    // For markdown-style blocks, find the end of the name before any space, bracket or other attribute
                    let name_end = trimmed[name_start..].find(|c: char| c == ' ' || c == ']' || c == '>')
                        .map(|pos| name_start + pos)
                        .unwrap_or(trimmed.len());
                    
                    current_question_name = Some(trimmed[name_start..name_end].trim().to_string());
                    println!("DEBUG: Found question block with name: {:?}", current_question_name);
                } else if let Some(name_start) = trimmed.find("name=\"") {
                    let name_start = name_start + 6; // skip "name=\""
                    // For XML-style blocks, find the closing quote for just the name attribute
                    let name_end = trimmed[name_start..].find('"')
                        .map(|pos| name_start + pos)
                        .unwrap_or(trimmed.len());
                    
                    current_question_name = Some(trimmed[name_start..name_end].trim().to_string());
                    println!("DEBUG: Found question block with XML name attribute: {:?}", current_question_name);
                }
            }
            
            // Check for question block end - handle both XML and markdown formats
            if (trimmed == "[/question]" || trimmed == "</meta:question>" || trimmed == "</question>") && in_question_block {
                in_question_block = false;
                
                // Store the question block info
                if let Some(name) = current_question_name.take() {
                    question_blocks.push((i, name.clone()));
                    println!("DEBUG: Recorded question block '{}' ending at line {}", name, i);
                } else {
                    println!("DEBUG: Found unnamed question block ending at line {}", i);
                }
            }
        }
        
        // Second pass: insert responses after their corresponding question blocks
        let mut i = 0;
        let mut response_blocks_added = 0;
        let mut unnamed_question_count = 0;
        
        while i < lines.len() {
            let line = lines[i];
            result.push_str(line);
            result.push('\n');
            
            // Check if this is the end of a question block - handle both XML and markdown formats
            let trimmed_line = line.trim();
            if trimmed_line == "[/question]" || trimmed_line == "</meta:question>" || trimmed_line == "</question>" {
                // Check if this is a named question block we identified
                let named_question_pos = question_blocks.iter().position(|(line_idx, _)| *line_idx == i);
                
                // Check if the next line is already a response block
                let next_is_response = i + 1 < lines.len() && 
                    (lines[i + 1].trim().starts_with("[response") || 
                     lines[i + 1].trim().starts_with("<meta:response"));
                println!("DEBUG: Next line is already a response block: {}", next_is_response);
                
                // If there's no response block following, try to add the corresponding one
                if !next_is_response {
                    if let Some(pos) = named_question_pos {
                        // Handle named question block
                        let (_, question_name) = &question_blocks[pos];
                        println!("DEBUG: Processing end of named question block '{}' at line {}", question_name, i);
                        
                        // Look for a response to this specific question in the outputs
                        // Try multiple possible formats for the response name
                        let response_name = format!("{}_response", question_name);
                        let error_response_name = format!("{}_error_response", question_name);
                        let response_results_name = format!("{}_results", question_name);
                        let response_dot_results_name = format!("{}.results", question_name);
                        
                        println!("DEBUG: Looking for response with names: '{}', '{}', '{}', '{}', or '{}'", 
                                 question_name, response_name, error_response_name, response_results_name, response_dot_results_name);
                        
                        // First check for error-response
                        let error_output = self.outputs.get(&error_response_name);
                        
                        if let Some(output) = error_output {
                            println!("DEBUG: Found matching error-response for '{}' (length: {})", question_name, output.len());
                            // Insert the error-response block after the question block
                            // Use the same format (XML or markdown) as the question block
                            if trimmed_line.starts_with("<") {
                                // XML format
                                result.push_str("  <meta:error-response>\n  ");
                                result.push_str(&output.replace("\n", "\n  ")); // Indent response content
                                result.push_str("\n  </meta:error-response>\n\n");
                            } else {
                                // Markdown format
                                result.push_str("[error-response]\n");
                                result.push_str(output);
                                result.push_str("\n[/error-response]\n\n");
                            }
                            response_blocks_added += 1;
                            println!("DEBUG: Added error-response block #{} for question '{}'", response_blocks_added, question_name);
                        } else {
                            // Try all possible regular response name formats
                            let output = self.outputs.get(&response_name)
                                .or_else(|| self.outputs.get(question_name))
                                .or_else(|| self.outputs.get(&response_results_name))
                                .or_else(|| self.outputs.get(&response_dot_results_name));
                            
                            if let Some(output) = output {
                                println!("DEBUG: Found matching response for '{}' (length: {})", question_name, output.len());
                                // Insert the response block after the question block
                                // Use the same format (XML or markdown) as the question block
                                if trimmed_line.starts_with("<") {
                                    // XML format
                                    result.push_str("  <meta:response>\n  ");
                                    result.push_str(&output.replace("\n", "\n  ")); // Indent response content
                                    result.push_str("\n  </meta:response>\n\n");
                                } else {
                                    // Markdown format
                                    result.push_str("[response]\n");
                                    result.push_str(output);
                                    result.push_str("\n[/response]\n\n");
                                }
                                response_blocks_added += 1;
                                println!("DEBUG: Added response block #{} for question '{}'", response_blocks_added, question_name);
                            } else {
                                println!("DEBUG: No matching response found for question '{}'", question_name);
                            }
                        }
                    } else {
                        // Handle unnamed question block
                        unnamed_question_count += 1;
                        println!("DEBUG: Processing end of unnamed question block #{} at line {}", unnamed_question_count, i);
                        
                        // First check for error-response for unnamed question blocks
                        if let Some(output) = self.outputs.get("question_error_response") {
                            println!("DEBUG: Found generic question_error_response (length: {})", output.len());
                            // Insert the error-response block after the question block
                            // Use the same format (XML or markdown) as the question block
                            if trimmed_line.starts_with("<") {
                                // XML format
                                result.push_str("  <meta:error-response>\n  ");
                                result.push_str(&output.replace("\n", "\n  ")); // Indent response content
                                result.push_str("\n  </meta:error-response>\n\n");
                            } else {
                                // Markdown format
                                result.push_str("[error-response]\n");
                                result.push_str(output);
                                result.push_str("\n[/error-response]\n\n");
                            }
                            response_blocks_added += 1;
                            println!("DEBUG: Added error-response block #{} for unnamed question", response_blocks_added);
                        } 
                        // For unnamed question blocks, check the generic "question_response" key
                        else if let Some(output) = self.outputs.get("question_response") {
                            println!("DEBUG: Found generic question_response (length: {})", output.len());
                            // Insert the response block after the question block
                            // Use the same format (XML or markdown) as the question block
                            if trimmed_line.starts_with("<") {
                                // XML format
                                result.push_str("  <meta:response>\n  ");
                                result.push_str(&output.replace("\n", "\n  ")); // Indent response content
                                result.push_str("\n  </meta:response>\n\n");
                            } else {
                                // Markdown format
                                result.push_str("[response]\n");
                                result.push_str(output);
                                result.push_str("\n[/response]\n\n");
                            }
                            response_blocks_added += 1;
                            println!("DEBUG: Added response block #{} for unnamed question", response_blocks_added);
                        } else {
                            println!("DEBUG: No generic question_response found for unnamed question");
                        }
                    }
                }
            }
            
            i += 1;
        }
        
        println!("DEBUG: Found {} named question blocks, {} unnamed question blocks, added {} response blocks", 
                 question_blocks.len(), unnamed_question_count, response_blocks_added);
        println!("DEBUG: Final document length: {}", result.len());
        
        Ok(result)
    }
    
    pub fn process_variable_references(&self, content: &str) -> String {
        let mut processed = content.to_string();

        // Process old ${variable} syntax using regex replacement
        let re_old = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
        processed = re_old.replace_all(&processed, |caps: &regex::Captures| {
            let var = &caps[1];
            if let Some(value) = self.outputs.get(var) {
                value.to_string()
            } else {
                caps[0].to_string()
            }
        }).to_string();

        // Process new <meta:reference .../> tags with target attribute
        let re = regex::Regex::new(r#"<meta:reference[^>]*target\s*=\s*["']([^"']+)["'][^>]*/>"#).unwrap();
        processed = re.replace_all(&processed, |caps: &regex::Captures| {
            let target = &caps[1];
            if let Some(val) = self.outputs.get(target) {
                val.to_string()
            } else {
                caps[0].to_string()
            }
        }).to_string();

        processed
    }
    
    pub fn apply_block_modifiers_to_variable(&self, block_name: &str, content: &str) -> String {
        if let Some(block) = self.blocks.get(block_name) {
            let trimmed = self.apply_trim(block, content);
            let truncated = self.apply_max_lines(block, &trimmed);
            truncated
        } else {
            content.to_string()
        }
    }
    
}
