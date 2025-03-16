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
    pub instance_id: String,
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
        
        // Process variable references in all blocks
        // We need to do this in a separate pass after all blocks are registered
        let mut blocks_to_update = Vec::new();
        
        for (name, block) in &self.blocks {
            // Process variable references in the content
            let processed_content = self.process_variable_references(&block.content);
            
            // Only update if the content actually changed
            if processed_content != block.content {
                blocks_to_update.push((name.clone(), processed_content));
            }
        }
        
        // Update the blocks and outputs with processed content
        for (name, processed_content) in blocks_to_update {
            // Update the block in the blocks map
            if let Some(block) = self.blocks.get_mut(&name) {
                block.content = processed_content.clone();
            }
            
            // Update the output in the outputs map if it's a data block
            if let Some(block) = self.blocks.get(&name) {
                if self.is_data_block(block) {
                    // Apply any modifiers to the data block content before storing
                    let modified_content = self.apply_block_modifiers_to_variable(&name, &processed_content);
                    self.outputs.insert(name.clone(), modified_content);
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
        let processed_content = self.process_variable_references(&block_content);
        let processed_content = self.process_conditional_blocks(&processed_content);
        
        // Execute based on block type
        let result = match block.block_type.as_str() {
            "code:python" => self.execute_python(&processed_content),
            "code:javascript" => self.execute_javascript(&processed_content),
            "code:rust" => self.execute_rust(&processed_content),
            "shell" => self.execute_shell(&processed_content),
            "api" => self.execute_api(&processed_content),
            "question" => self.execute_question(&block, &processed_content),
            _ => Ok(processed_content),
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
    
    // Process variable references like ${block_name} or ${block_name:fallback_value}
    // Also handles enhanced variable references with modifiers like ${block_name:format=markdown}
    pub fn process_variable_references(&self, content: &str) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Processing variable references in: '{}'", 
                     if content.len() > 100 { &content[..100] } else { content });
        }
        
        // Check if we should preserve variable references in original block content
        // This is determined by environment variable or context
        let preserve_refs = std::env::var("LLM_PRESERVE_REFS").unwrap_or_default() == "1" || 
                           std::env::var("LLM_PRESERVE_REFS").unwrap_or_default().to_lowercase() == "true";
        
        // Also check if the content contains a format modifier, which should be preserved
        let contains_format_modifier = content.contains("${") && 
                                      (content.contains(":format=") || 
                                       content.contains(":transform=") || 
                                       content.contains(":highlight"));
        
        if preserve_refs || contains_format_modifier {
            if debug_enabled {
                println!("DEBUG: Preserving original variable references in block content");
                if contains_format_modifier {
                    println!("DEBUG: Content contains format modifiers that should be preserved");
                }
            }
            return content.to_string();
        }
        
        let result = self.process_variable_references_internal(content, &mut Vec::new());
        
        if debug_enabled && result != content {
            println!("DEBUG: Variable references resolved. Result: '{}'", 
                     if result.len() > 100 { &result[..100] } else { &result });
        }
        
        result
    }
    
    // Helper function to look up a variable value, handling dotted names
    fn lookup_variable(&self, var_name: &str) -> Option<String> {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("lookup_variable called with: '{}'", var_name);
        }
        
        // Check if we're looking for a property using dot notation (e.g., user-data.name)
        if var_name.contains('.') {
            let parts: Vec<&str> = var_name.splitn(2, '.').collect();
            let base_var = parts[0];
            let property_path = parts[1];
            
            if debug_enabled {
                println!("Looking up property: base='{}', property='{}'", base_var, property_path);
            }
            
            // Get the base variable
            if let Some(base_value) = self.outputs.get(base_var) {
                // Try to parse as JSON
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(base_value) {
                    // Navigate the JSON path
                    let mut current = &json_value;
                    for key in property_path.split('.') {
                        if let Some(next) = current.get(key) {
                            current = next;
                        } else {
                            if debug_enabled {
                                println!("JSON property not found: {}", key);
                            }
                            return None;
                        }
                    }
                    
                    // Convert the found value to string
                    match current {
                        serde_json::Value::String(s) => return Some(s.clone()),
                        serde_json::Value::Number(n) => return Some(n.to_string()),
                        serde_json::Value::Bool(b) => return Some(b.to_string()),
                        serde_json::Value::Null => return Some("null".to_string()),
                        _ => return Some(current.to_string()),
                    }
                } else if debug_enabled {
                    println!("Base variable is not valid JSON: {}", base_var);
                }
            }
            
            // If not found as JSON property, try common suffixes
            let block_name = parts[0];
            let suffix = parts[1];
            
            // Handle common suffixes
            if suffix == "results" {
                let results_key = format!("{}_results", block_name);
                
                if debug_enabled {
                    println!("  Looking up results_key: '{}'", results_key);
                }
                
                if let Some(value) = self.outputs.get(&results_key) {
                    return Some(value.clone());
                }
            } else if suffix == "error" {
                let error_key = format!("{}_error", block_name);
                
                if debug_enabled {
                    println!("  Looking up error_key: '{}'", error_key);
                }
                
                if let Some(value) = self.outputs.get(&error_key) {
                    return Some(value.clone());
                }
            }
            
            return None;
        }
        
        // Direct lookup for simple variable names
        if let Some(value) = self.outputs.get(var_name) {
            if debug_enabled {
                println!("  Direct lookup succeeded for '{}'", var_name);
            }
            return Some(value.clone());
        }
        
        // If not found in outputs, check if there's a block with this name
        if let Some(block) = self.blocks.get(var_name) {
            // For blocks, return their content
            return Some(block.content.clone());
        }
        
        // Check fallbacks
        self.fallbacks.get(var_name).cloned()
    }
    
    // Internal implementation that tracks processing variables to detect circular references
    fn process_variable_references_internal(&self, content: &str, processing_vars: &mut Vec<String>) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("process_variable_references_internal called with: '{}'", 
                     if content.len() > 100 { &content[..100] } else { content });
            println!("Current processing_vars: {:?}", processing_vars);
        }
        
        // Regular expression to find variable references like ${variable_name}
        let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
        
        let mut result = content.to_string();
        let mut last_end = 0;
        let mut processed_content = String::new();
        
        for cap in re.captures_iter(content) {
            let whole_match = cap.get(0).unwrap();
            let var_ref = cap.get(1).unwrap().as_str();
            
            if debug_enabled {
                println!("Found variable reference: ${{{}}}", var_ref);
            }
            
            // First, check if the variable reference itself contains nested references
            // Process any nested variable references in the variable reference itself
            let processed_var_ref = self.process_variable_references_internal(var_ref, &mut processing_vars.clone());
            
            if processed_var_ref != var_ref {
                if debug_enabled {
                    println!("Processed nested variable reference: '{}' -> '{}'", var_ref, processed_var_ref);
                }
                
                // Extract variable name and modifiers from the processed reference
                let (var_name, modifiers) = self.parse_variable_reference(&processed_var_ref);
                
                // Check for circular references
                if processing_vars.contains(&var_name) {
                    println!("WARNING: Circular reference detected for variable: {}", var_name);
                    processed_content.push_str(&content[last_end..whole_match.start()]);
                    processed_content.push_str(&format!("${{CIRCULAR_REFERENCE:{}}}", var_name));
                    last_end = whole_match.end();
                    continue;
                }
                
                // Look up the variable value
                let replacement = if let Some(value) = self.lookup_variable(&var_name) {
                    // Add this variable to the processing stack to detect circular references
                    processing_vars.push(var_name.clone());
                    
                    // Process any nested variable references in the value
                    let processed_value = self.process_variable_references_internal(&value, processing_vars);
                    
                    // Remove this variable from the processing stack
                    processing_vars.pop();
                    
                    // Apply modifiers to the value
                    self.apply_enhanced_modifiers(&var_name, &processed_value, &modifiers)
                } else {
                    // Variable not found, check for fallback
                    if let Some(fallback) = modifiers.get("fallback") {
                        fallback.clone()
                    } else if let Some(fallback) = self.fallbacks.get(&var_name) {
                        fallback.clone()
                    } else {
                        // No fallback specified
                        if debug_enabled {
                            println!("Variable not found and no fallback: {}", var_name);
                        }
                        format!("${{UNDEFINED:{}}}", var_name)
                    }
                };
                
                processed_content.push_str(&content[last_end..whole_match.start()]);
                processed_content.push_str(&replacement);
                last_end = whole_match.end();
            } else {
                // No nested references, process normally
                // Extract variable name and modifiers
                let (var_name, modifiers) = self.parse_variable_reference(var_ref);
                
                // Check for circular references
                if processing_vars.contains(&var_name) {
                    println!("WARNING: Circular reference detected for variable: {}", var_name);
                    processed_content.push_str(&content[last_end..whole_match.start()]);
                    processed_content.push_str(&format!("${{CIRCULAR_REFERENCE:{}}}", var_name));
                    last_end = whole_match.end();
                    continue;
                }
                
                // Look up the variable value
                let replacement = if let Some(value) = self.lookup_variable(&var_name) {
                    // Add this variable to the processing stack to detect circular references
                    processing_vars.push(var_name.clone());
                    
                    // Process any nested variable references in the value
                    let processed_value = self.process_variable_references_internal(&value, processing_vars);
                    
                    // Remove this variable from the processing stack
                    processing_vars.pop();
                    
                    // Apply modifiers to the value
                    self.apply_enhanced_modifiers(&var_name, &processed_value, &modifiers)
                } else {
                    // Variable not found, check for fallback
                    if let Some(fallback) = modifiers.get("fallback") {
                        fallback.clone()
                    } else if let Some(fallback) = self.fallbacks.get(&var_name) {
                        fallback.clone()
                    } else {
                        // No fallback specified
                        if debug_enabled {
                            println!("Variable not found and no fallback: {}", var_name);
                        }
                        format!("${{UNDEFINED:{}}}", var_name)
                    }
                };
                
                processed_content.push_str(&content[last_end..whole_match.start()]);
                processed_content.push_str(&replacement);
                last_end = whole_match.end();
            }
        }
        
        // Add any remaining content
        if last_end < content.len() {
            processed_content.push_str(&content[last_end..]);
        }
        
        processed_content
    }
    
    // Parse a variable reference into name and modifiers
    fn parse_variable_reference(&self, var_ref: &str) -> (String, HashMap<String, String>) {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Parsing variable reference: '{}'", var_ref);
        }
        
        let mut modifiers = HashMap::new();
        
        // Check if the variable reference contains modifiers
        if var_ref.contains(':') {
            let parts: Vec<&str> = var_ref.splitn(2, ':').collect();
            let var_name = parts[0].to_string();
            
            // Parse modifiers (format: key=value,key2=value2)
            if parts.len() > 1 {
                let modifier_str = parts[1];
                
                if debug_enabled {
                    println!("DEBUG: Found modifiers: '{}'", modifier_str);
                }
                
                // Simple fallback value without key
                if !modifier_str.contains('=') && !modifier_str.contains(',') {
                    modifiers.insert("fallback".to_string(), modifier_str.to_string());
                    
                    if debug_enabled {
                        println!("DEBUG: Simple fallback value: '{}'", modifier_str);
                    }
                    
                    return (var_name, modifiers);
                }
                
                // Parse key-value modifiers with support for nested structures
                let mut current_pos = 0;
                let mut nesting_level = 0;
                let mut in_quotes = false;
                let mut start_pos = 0;
                let mut escape_next = false;
                
                for (i, c) in modifier_str.char_indices() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }
                    
                    match c {
                        '\\' => escape_next = true,
                        '"' => in_quotes = !in_quotes,
                        '(' | '[' | '{' if !in_quotes => nesting_level += 1,
                        ')' | ']' | '}' if !in_quotes => {
                            if nesting_level > 0 {
                                nesting_level -= 1;
                            }
                        },
                        ',' if !in_quotes && nesting_level == 0 => {
                            // Found a comma outside quotes and nested structures
                            let single_modifier = &modifier_str[start_pos..i].trim();
                            if !single_modifier.is_empty() {
                                self.parse_single_modifier(single_modifier, &mut modifiers);
                            }
                            start_pos = i + 1;
                        },
                        _ => {}
                    }
                    current_pos = i + 1;
                }
                
                // Process the last modifier
                if start_pos < current_pos {
                    let single_modifier = &modifier_str[start_pos..current_pos].trim();
                    if !single_modifier.is_empty() {
                        self.parse_single_modifier(single_modifier, &mut modifiers);
                    }
                }
                
                // Handle special shorthand modifiers
                if modifiers.contains_key("highlight") && modifiers.get("highlight").unwrap() == "true" {
                    // If highlight is true without a language, try to detect from block type
                    if let Some(block) = self.blocks.get(&var_name) {
                        if block.block_type.starts_with("code:") {
                            let lang = block.block_type.split(':').nth(1).unwrap_or("text");
                            modifiers.insert("highlight".to_string(), lang.to_string());
                        }
                    }
                }
                
                // Handle format=markdown shorthand
                if modifiers.contains_key("markdown") && modifiers.get("markdown").unwrap() == "true" {
                    modifiers.insert("format".to_string(), "markdown".to_string());
                }
                
                // Handle format=json shorthand
                if modifiers.contains_key("json") && modifiers.get("json").unwrap() == "true" {
                    modifiers.insert("format".to_string(), "json".to_string());
                }
            }
            
            if debug_enabled {
                println!("DEBUG: Parsed {} modifiers for variable '{}'", modifiers.len(), var_name);
                for (k, v) in &modifiers {
                    println!("DEBUG:   '{}' = '{}'", k, v);
                }
            }
            
            return (var_name, modifiers);
        }
        
        // No modifiers
        (var_ref.to_string(), modifiers)
    }
    
    /// Parse a single modifier in the format key=value
    fn parse_single_modifier(&self, modifier: &str, modifiers: &mut HashMap<String, String>) {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Parsing single modifier: '{}'", modifier);
        }
        
        if let Some(pos) = modifier.find('=') {
            let key = modifier[..pos].trim().to_string();
            let raw_value = modifier[pos+1..].trim();
            
            // Handle quoted values
            let value = if (raw_value.starts_with('"') && raw_value.ends_with('"')) || 
                          (raw_value.starts_with('\'') && raw_value.ends_with('\'')) {
                // Remove the quotes
                let quote_char = raw_value.chars().next().unwrap();
                let mut chars = raw_value.chars();
                chars.next(); // Skip first quote
                chars.next_back(); // Skip last quote
                chars.collect::<String>()
            } else {
                raw_value.to_string()
            };
            
            if debug_enabled {
                println!("DEBUG:   Key-value pair: '{}' = '{}'", key, value);
            }
            
            modifiers.insert(key, value);
        } else {
            // Handle boolean flags without values
            let key = modifier.trim().to_string();
            
            if debug_enabled {
                println!("DEBUG:   Boolean flag: '{}'", key);
            }
            
            modifiers.insert(key, "true".to_string());
        }
    }
    
    // Process conditional blocks like ${if:condition_var}content${endif}
    fn process_conditional_blocks(&self, content: &str) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: process_conditional_blocks called with content length: {}", content.len());
        }
        
        let mut result = content.to_string();
        
        // Find all conditional blocks
        let if_pattern = r"\$\{if:([^}]+)\}(.*?)\$\{endif\}";
        let re = regex::Regex::new(if_pattern).unwrap();
        
        // Process each conditional block
        while let Some(captures) = re.captures(&result) {
            let full_match = captures.get(0).unwrap().as_str();
            let condition_var = captures.get(1).unwrap().as_str();
            let conditional_content = captures.get(2).unwrap().as_str();
            
            if debug_enabled {
                println!("DEBUG: Found conditional block with condition: {}", condition_var);
            }
            
            // Process any variable references in the condition itself
            let processed_condition = self.process_variable_references(condition_var);
            
            if debug_enabled && processed_condition != condition_var {
                println!("DEBUG: Processed condition: '{}' -> '{}'", condition_var, processed_condition);
            }
            
            // Evaluate the condition
            let condition_met = self.evaluate_condition(&processed_condition);
            
            if debug_enabled {
                println!("DEBUG: Condition '{}' evaluated to: {}", processed_condition, condition_met);
            }
            
            // Process the conditional content for nested variables
            let processed_content = self.process_variable_references(conditional_content);
            
            // Replace the conditional block based on the condition
            if condition_met {
                if debug_enabled {
                    println!("DEBUG: Condition met, including content");
                }
                result = result.replace(full_match, &processed_content);
            } else {
                if debug_enabled {
                    println!("DEBUG: Condition not met, excluding content");
                }
                result = result.replace(full_match, "");
            }
        }
        
        // Also handle else blocks: ${if:condition}content${else}alternative${endif}
        let if_else_pattern = r"\$\{if:([^}]+)\}(.*?)\$\{else\}(.*?)\$\{endif\}";
        let re_else = regex::Regex::new(if_else_pattern).unwrap();
        
        // Process each if-else block
        while let Some(captures) = re_else.captures(&result) {
            let full_match = captures.get(0).unwrap().as_str();
            let condition_var = captures.get(1).unwrap().as_str();
            let if_content = captures.get(2).unwrap().as_str();
            let else_content = captures.get(3).unwrap().as_str();
            
            if debug_enabled {
                println!("DEBUG: Found if-else block with condition: {}", condition_var);
            }
            
            // Process any variable references in the condition itself
            let processed_condition = self.process_variable_references(condition_var);
            
            // Evaluate the condition
            let condition_met = self.evaluate_condition(&processed_condition);
            
            if debug_enabled {
                println!("DEBUG: If-else condition '{}' evaluated to: {}", processed_condition, condition_met);
            }
            
            // Process both content blocks for nested variables
            let processed_if_content = self.process_variable_references(if_content);
            let processed_else_content = self.process_variable_references(else_content);
            
            // Replace the conditional block based on the condition
            if condition_met {
                if debug_enabled {
                    println!("DEBUG: If-else condition met, including 'if' content");
                }
                result = result.replace(full_match, &processed_if_content);
            } else {
                if debug_enabled {
                    println!("DEBUG: If-else condition not met, including 'else' content");
                }
                result = result.replace(full_match, &processed_else_content);
            }
        }
        
        result
    }
    
    // Apply enhanced modifiers to variable values
    fn apply_enhanced_modifiers(&self, var_name: &str, value: &str, modifiers: &HashMap<String, String>) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: apply_enhanced_modifiers for '{}' with {} modifiers", 
                     var_name, modifiers.len());
            for (k, v) in modifiers {
                println!("DEBUG:   '{}' = '{}'", k, v);
            }
        }
        
        // If there are no modifiers, return the original value
        if modifiers.is_empty() {
            return value.to_string();
        }
        
        let mut result = value.to_string();
        
        // Apply modifiers in a specific order for predictable results
        
        // 1. Apply transformations first (uppercase, lowercase, substring, etc.)
        if let Some(transform) = modifiers.get("transform") {
            if debug_enabled {
                println!("DEBUG: Applying transform modifier: {}", transform);
            }
            
            match transform.as_str() {
                "uppercase" => {
                    result = result.to_uppercase();
                },
                "lowercase" => {
                    result = result.to_lowercase();
                },
                "capitalize" => {
                    let mut new_result = String::new();
                    let mut capitalize_next = true;
                    
                    for c in result.chars() {
                        if c.is_alphabetic() {
                            if capitalize_next {
                                new_result.extend(c.to_uppercase());
                                capitalize_next = false;
                            } else {
                                new_result.push(c);
                            }
                        } else {
                            new_result.push(c);
                            if c.is_whitespace() || c == '.' || c == '!' || c == '?' {
                                capitalize_next = true;
                            }
                        }
                    }
                    result = new_result;
                },
                transform_str if transform_str.starts_with("substring(") && transform_str.ends_with(")") => {
                    // Extract parameters from substring(start,end)
                    let params_str = transform_str.trim_start_matches("substring(").trim_end_matches(")");
                    let params: Vec<&str> = params_str.split(',').collect();
                    
                    if params.len() == 2 {
                        if let (Ok(start), Ok(end)) = (params[0].trim().parse::<usize>(), params[1].trim().parse::<usize>()) {
                            if start < result.len() {
                                let end = std::cmp::min(end, result.len());
                                result = result[start..end].to_string();
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        // 2. Apply regex replacements if specified
        if let Some(regex_pattern) = modifiers.get("regex") {
            if let Some(replacement) = modifiers.get("replacement") {
                if debug_enabled {
                    println!("DEBUG: Applying regex replacement: {} -> {}", regex_pattern, replacement);
                }
                
                match regex::Regex::new(regex_pattern) {
                    Ok(re) => {
                        result = re.replace_all(&result, replacement).to_string();
                    },
                    Err(e) => {
                        if debug_enabled {
                            println!("DEBUG: Invalid regex pattern: {} - {}", regex_pattern, e);
                        }
                    }
                }
            }
        }
        
        // 3. Limit modifier
        if let Some(limit_str) = modifiers.get("limit") {
            if debug_enabled {
                println!("DEBUG: Applying limit modifier: {}", limit_str);
            }
            
            if let Ok(limit) = limit_str.parse::<usize>() {
                // Limit by number of lines
                let lines: Vec<&str> = result.lines().collect();
                if lines.len() > limit {
                    result = lines.iter().take(limit).cloned().collect::<Vec<&str>>().join("\n");
                    result.push_str("\n...(truncated)");
                }
            }
        }
        
        // 4. Format modifier (markdown, json, code, plain)
        if let Some(format) = modifiers.get("format") {
            if debug_enabled {
                println!("DEBUG: Applying format modifier: {}", format);
            }
            result = self.apply_format_modifier(&result, format);
        }
        
        // 5. Highlighting for code blocks
        if let Some(highlight) = modifiers.get("highlight") {
            if debug_enabled {
                println!("DEBUG: Applying highlight modifier: {}", highlight);
            }
            
            if highlight == "true" {
                // Auto-detect language
                if let Some(block) = self.blocks.get(var_name) {
                    if block.block_type.starts_with("code:") {
                        let lang = block.block_type.split(':').nth(1).unwrap_or("text");
                        result = format!("```{}\n{}\n```", lang, result);
                    } else {
                        result = format!("```\n{}\n```", result);
                    }
                } else {
                    result = format!("```\n{}\n```", result);
                }
            } else {
                // Use specified language
                result = format!("```{}\n{}\n```", highlight, result);
            }
        }
        
        // 6. Include modifiers for code and results
        if modifiers.get("include_code").map_or(false, |v| v == "true") {
            if debug_enabled {
                println!("DEBUG: Applying include_code modifier");
            }
            
            // Include the code content
            if let Some(code_block) = self.blocks.get(var_name) {
                // Determine language for syntax highlighting
                let language = if code_block.block_type.starts_with("code:") {
                    code_block.block_type.split(':').nth(1).unwrap_or("text")
                } else if code_block.block_type == "shell" {
                    "bash"
                } else {
                    "text"
                };
                
                // Format the code with proper syntax highlighting
                let code_content = code_block.content.clone();
                let formatted_code = format!("### Code\n\n```{}\n{}\n```", language, code_content);
                
                // If we already have content, prepend the code
                if !result.is_empty() {
                    result = format!("{}\n\n{}", formatted_code, result);
                } else {
                    result = formatted_code;
                }
            }
        }
        
        if modifiers.get("include_results").map_or(false, |v| v == "true") {
            if debug_enabled {
                println!("DEBUG: Applying include_results modifier");
            }
            
            // Include the results for this code block
            let results_key = format!("{}_results", var_name);
            let dot_results_key = format!("{}.results", var_name);
            
            if let Some(results) = self.outputs.get(&results_key).or_else(|| self.outputs.get(&dot_results_key)) {
                // Format the results
                let formatted_results = format!("### Results\n\n```\n{}\n```", results);
                
                // If we already have content, append the results
                if !result.is_empty() {
                    result = format!("{}\n\n{}", result, formatted_results);
                } else {
                    result = formatted_results;
                }
            }
        }
        
        // 7. Include sensitive data conditionally
        if let Some(include_condition) = modifiers.get("include_sensitive") {
            if debug_enabled {
                println!("DEBUG: Applying include_sensitive modifier: {}", include_condition);
            }
            
            // Check if the condition is true
            let include = if include_condition.starts_with("${") && include_condition.ends_with("}") {
                // This is a variable reference
                let var_name = include_condition.trim_start_matches("${").trim_end_matches("}");
                if let Some(value) = self.lookup_variable(var_name) {
                    value == "true" || value == "1" || value == "yes" || value == "on"
                } else {
                    false
                }
            } else {
                include_condition == "true" || include_condition == "1" || include_condition == "yes" || include_condition == "on"
            };
            
            if !include {
                // Remove sensitive information
                if let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                    if let Some(obj) = json_value.as_object_mut() {
                        // Define sensitive field patterns
                        let sensitive_fields = [
                            "password", "passwd", "secret", "token", "api_key", "apikey", 
                            "private_key", "privatekey", "sensitive", "credential", 
                            "auth", "authentication", "key", "cert", "certificate"
                        ];
                        
                        // Remove all fields that match sensitive patterns
                        let keys_to_remove: Vec<String> = obj.keys()
                            .filter(|k| {
                                let k_lower = k.to_lowercase();
                                sensitive_fields.iter().any(|&pattern| k_lower.contains(pattern))
                            })
                            .cloned()
                            .collect();
                        
                        for key in keys_to_remove {
                            obj.remove(&key);
                        }
                        
                        // Also recursively check nested objects
                        self.redact_sensitive_fields(obj);
                        
                        if let Ok(filtered) = serde_json::to_string_pretty(&json_value) {
                            result = filtered;
                        }
                    }
                }
            }
        }
        
        // 8. JSON path extraction if specified
        if let Some(json_path) = modifiers.get("json_path") {
            if debug_enabled {
                println!("DEBUG: Applying JSON path extraction: {}", json_path);
            }
            
            result = self.extract_json_path(&result, json_path);
        }
        
        // 9. Preview modifier
        if modifiers.get("preview").map_or(false, |v| v == "true") {
            if debug_enabled {
                println!("DEBUG: Applying preview modifier");
            }
            
            // Create a preview of the content (first few lines or characters)
            let lines: Vec<&str> = result.lines().collect();
            if lines.len() > 5 {
                result = lines.iter().take(5).cloned().collect::<Vec<&str>>().join("\n") + "\n...";
            } else if result.len() > 200 {
                result = result[..200].to_string() + "...";
            }
        }
        
        // 10. Apply trim operations
        if let Some(trim_type) = modifiers.get("trim") {
            if debug_enabled {
                println!("DEBUG: Applying trim modifier: {}", trim_type);
            }
            
            match trim_type.as_str() {
                "true" | "yes" | "1" | "both" => result = result.trim().to_string(),
                "start" | "left" => result = result.trim_start().to_string(),
                "end" | "right" => result = result.trim_end().to_string(),
                "lines" => {
                    // Trim each line individually
                    result = result.lines()
                        .map(|line| line.trim())
                        .collect::<Vec<&str>>()
                        .join("\n");
                },
                "empty_lines" => {
                    // Remove empty lines
                    result = result.lines()
                        .filter(|line| !line.trim().is_empty())
                        .collect::<Vec<&str>>()
                        .join("\n");
                },
                _ => {}
            }
        }
        
        // Apply standard modifiers from the block
        if let Some(block) = self.blocks.get(var_name) {
            if debug_enabled {
                println!("DEBUG: Applying standard block modifiers");
            }
            result = self.apply_modifiers_to_variable(var_name, &result, &HashMap::new());
        }
        
        if debug_enabled {
            println!("DEBUG: Final result after applying modifiers (length: {})", result.len());
        }
        
        result
    }
    
    // Apply format modifier to content
    fn apply_format_modifier(&self, content: &str, format: &str) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying format '{}' to content (length: {})", format, content.len());
        }
        
        match format {
            "markdown" => {
                // Check if content is already in markdown format
                if content.contains("#") || content.contains("```") || 
                   content.contains("**") || content.contains("__") {
                    if debug_enabled {
                        println!("DEBUG: Content appears to already be in markdown format");
                    }
                    return content.to_string();
                }
                
                // Convert JSON to markdown if possible
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
                    if debug_enabled {
                        println!("DEBUG: Converting JSON to markdown");
                    }
                    
                    let mut markdown = String::new();
                    
                    if let Some(obj) = json_value.as_object() {
                        markdown.push_str("## Data\n\n");
                        for (key, value) in obj {
                            markdown.push_str(&format!("**{}**: ", key));
                            
                            match value {
                                serde_json::Value::Array(arr) => {
                                    markdown.push_str("\n");
                                    for item in arr.iter() {
                                        markdown.push_str(&format!("- {}\n", item));
                                    }
                                },
                                _ => {
                                    markdown.push_str(&format!("{}\n", value));
                                }
                            }
                        }
                    } else if let Some(arr) = json_value.as_array() {
                        markdown.push_str("## Items\n\n");
                        for item in arr {
                            markdown.push_str(&format!("- {}\n", item));
                        }
                    }
                    
                    return markdown;
                }
                
                // If not JSON, try to format as markdown based on content structure
                let lines: Vec<&str> = content.lines().collect();
                
                // If it looks like a table (contains tab or multiple spaces)
                if lines.len() > 1 && lines.iter().all(|line| line.contains('\t') || line.contains("  ")) {
                    if debug_enabled {
                        println!("DEBUG: Converting tabular data to markdown table");
                    }
                    
                    let mut markdown = String::new();
                    let mut is_first_line = true;
                    
                    for line in lines {
                        let columns: Vec<&str> = line.split('\t')
                            .map(|s| s.trim())
                            .collect();
                        
                        // Add columns as markdown table
                        markdown.push_str("| ");
                        markdown.push_str(&columns.join(" | "));
                        markdown.push_str(" |\n");
                        
                        // Add separator after header
                        if is_first_line {
                            markdown.push_str("| ");
                            markdown.push_str(&columns.iter()
                                .map(|_| "---")
                                .collect::<Vec<&str>>()
                                .join(" | "));
                            markdown.push_str(" |\n");
                            is_first_line = false;
                        }
                    }
                    
                    return markdown;
                }
                
                // Simple text to markdown conversion
                let mut markdown = String::new();
                
                // Add a title if the first line looks like a title
                if !lines.is_empty() {
                    markdown.push_str(&format!("## {}\n\n", lines[0]));
                    
                    // Add the rest as paragraphs
                    let mut in_paragraph = false;
                    
                    for line in &lines[1..] {
                        if line.trim().is_empty() {
                            if in_paragraph {
                                markdown.push_str("\n\n");
                                in_paragraph = false;
                            }
                        } else {
                            markdown.push_str(line);
                            markdown.push_str(" ");
                            in_paragraph = true;
                        }
                    }
                } else {
                    // Just return the content as-is
                    markdown = content.to_string();
                }
                
                markdown
            },
            "json" => {
                // Try to format as JSON if it's not already
                if !content.trim().starts_with('{') && !content.trim().starts_with('[') {
                    if debug_enabled {
                        println!("DEBUG: Content doesn't appear to be JSON, returning as-is");
                    }
                    return content.to_string();
                }
                
                // Try to pretty-print JSON
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
                    if debug_enabled {
                        println!("DEBUG: Pretty-printing JSON");
                    }
                    
                    if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
                        return pretty;
                    }
                }
                
                if debug_enabled {
                    println!("DEBUG: Failed to parse as JSON, returning as-is");
                }
                
                content.to_string()
            },
            "plain" => content.to_string(),
            "code" | "codeblock" => {
                // Check if content is already in a code block
                if content.trim().starts_with("```") && content.trim().ends_with("```") {
                    return content.to_string();
                }
                format!("```\n{}\n```", content)
            },
            "python" => format!("```python\n{}\n```", content),
            "javascript" | "js" => format!("```javascript\n{}\n```", content),
            "rust" => format!("```rust\n{}\n```", content),
            "bash" | "shell" => format!("```bash\n{}\n```", content),
            "bold" => format!("**{}**", content),
            "italic" => format!("*{}*", content),
            "html" => {
                // Convert to HTML if it's markdown or plain text
                if content.contains("#") || content.contains("```") || 
                   content.contains("**") || content.contains("__") {
                    if debug_enabled {
                        println!("DEBUG: Converting markdown to HTML");
                    }
                    
                    // Simple markdown to HTML conversion
                    let mut html = String::new();
                    
                    for line in content.lines() {
                        if line.starts_with("# ") {
                            html.push_str(&format!("<h1>{}</h1>\n", &line[2..]));
                        } else if line.starts_with("## ") {
                            html.push_str(&format!("<h2>{}</h2>\n", &line[3..]));
                        } else if line.starts_with("### ") {
                            html.push_str(&format!("<h3>{}</h3>\n", &line[4..]));
                        } else if line.starts_with("- ") {
                            html.push_str(&format!("<li>{}</li>\n", &line[2..]));
                        } else if line.trim().is_empty() {
                            html.push_str("<br>\n");
                        } else {
                            html.push_str(&format!("<p>{}</p>\n", line));
                        }
                    }
                    
                    return html;
                }
                
                // If it's not markdown, wrap in HTML paragraph tags
                format!("<p>{}</p>", content.replace("\n", "<br>\n"))
            },
            _ => content.to_string()
        }
    }
    
    // Execute different types of blocks
    
    pub fn execute_python(&self, code: &str) -> Result<String, ExecutorError> {
        // Debug: Print original code
        println!("DEBUG: Original Python code:\n{}", code);
        
        // Preprocess the code to handle JSON data
        let processed_code = self.preprocess_python_code(code);
        
        // Debug: Print processed code
        println!("DEBUG: Processed Python code:\n{}", processed_code);
        
        // Find Python interpreter by trying different commands/paths
        let python_commands = vec!["python3", "python", "py"];
        let python_paths = vec![
            "/usr/bin/python3",
            "/usr/bin/python",
            "/usr/local/bin/python3",
            "/usr/local/bin/python",
        ];
        
        // First try commands that should be in PATH
        let mut python_cmd = None;
        for cmd in &python_commands {
            println!("DEBUG: Trying Python command: '{}'", cmd);
            if Command::new(cmd)
                .arg("--version")
                .output()
                .is_ok() {
                python_cmd = Some(cmd.to_string());
                println!("DEBUG: Found working Python command: '{}'", cmd);
                break;
            }
        }
        
        // If no command worked, try specific paths
        if python_cmd.is_none() {
            for path in &python_paths {
                println!("DEBUG: Trying Python path: '{}'", path);
                if std::path::Path::new(path).exists() {
                    if Command::new(path)
                        .arg("--version")
                        .output()
                        .is_ok() {
                        python_cmd = Some(path.to_string());
                        println!("DEBUG: Found working Python path: '{}'", path);
                        break;
                    }
                }
            }
        }
        
        // If we still don't have a working Python, return an error
        let python_cmd = match python_cmd {
            Some(cmd) => cmd,
            None => return Err(ExecutorError::ExecutionFailed(
                "Could not find a working Python interpreter. Please ensure Python is installed and in your PATH.".to_string()
            )),
        };
        
        println!("DEBUG: Executing Python with '{}'", python_cmd);
        
        let result = Command::new(&python_cmd)
            .arg("-c")
            .arg(&processed_code)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
            
        match result {
            Ok(child) => {
                // Successfully spawned the process, now get the output
                match child.wait_with_output() {
                    Ok(output) => {
                        if output.status.success() {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            println!("DEBUG: Python execution succeeded with output:\n{}", stdout);
                            return Ok(stdout);
                        } else {
                            // Command executed but returned an error
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            println!("DEBUG: Python execution failed with error:\n{}", stderr);
                            
                            // Provide more detailed error message
                            let error_msg = if !stderr.is_empty() {
                                format!("Python execution error:\n{}", stderr)
                            } else if !stdout.is_empty() {
                                format!("Python execution failed with output:\n{}", stdout)
                            } else {
                                "Python execution failed with no error output".to_string()
                            };
                            
                            return Err(ExecutorError::ExecutionFailed(error_msg));
                        }
                    },
                    Err(e) => {
                        println!("DEBUG: Failed to get output from Python process: {}", e);
                        return Err(ExecutorError::ExecutionFailed(
                            format!("Failed to get output from Python process: {}", e)
                        ));
                    }
                }
            },
            Err(e) => {
                // Command not found or other spawn error
                println!("DEBUG: Error spawning Python process: {}", e);
                return Err(ExecutorError::ExecutionFailed(
                    format!("Failed to execute Python code with '{}'. Error: {}\nPlease ensure Python is installed and in your PATH.", python_cmd, e)
                ));
            }
        }
    }
    
    // Helper function to preprocess Python code for JSON handling
    fn preprocess_python_code(&self, code: &str) -> String {
        // Always import json and ast at the beginning
        let mut processed = String::from("import json\nimport ast\n");
        
        // Process each line to detect and convert JSON strings to Python objects
        for line in code.lines() {
            let mut processed_line = line.to_string();
            
            // Debug: Print the line being processed
            println!("DEBUG: Processing line: '{}'", line);
            
            // Look for variable assignments with JSON-like content
            if let Some(pos) = line.find('=') {
                let var_name = line[..pos].trim();
                let value = line[pos+1..].trim();
                
                println!("DEBUG: Found assignment: var_name='{}', value='{}'", var_name, value);
                
                // Check if the value looks like a JSON array or object
                if (value.starts_with('[') && value.ends_with(']')) || 
                   (value.starts_with('{') && value.ends_with('}')) {
                    println!("DEBUG: Detected JSON-like structure in: '{}'", value);
                    
                    // Use ast.literal_eval for Python literals (safer than eval, handles arrays better than json.loads)
                    processed_line = format!("try:\n    {} = ast.literal_eval('''{}''')\nexcept (SyntaxError, ValueError):\n    try:\n        {} = json.loads('''{}''')\n    except json.JSONDecodeError:\n        {} = '''{}'''", 
                        var_name, value, var_name, value, var_name, value);
                    
                    println!("DEBUG: Transformed to: '{}'", processed_line);
                }
            }
            
            processed.push_str(&processed_line);
            processed.push('\n');
        }
        
        processed
    }
    
    pub fn execute_javascript(&self, code: &str) -> Result<String, ExecutorError> {
        let child = Command::new("node")
            .arg("-e")
            .arg(code)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
            
        let output = child.wait_with_output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }
    
    pub fn execute_rust(&self, code: &str) -> Result<String, ExecutorError> {
        // Create a temporary file with the code
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("temp.rs");
        
        let mut file = File::create(&file_path)?;
        file.write_all(code.as_bytes())?;
        file.flush()?;
        
        // Compile and run with rustc
        let output = Command::new("rustc")
            .arg(&file_path)
            .arg("-o")
            .arg(temp_dir.path().join("temp"))
            .output()?;
            
        if !output.status.success() {
            return Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        let output = Command::new(temp_dir.path().join("temp"))
            .output()?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
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
    pub fn generate_results_block(&self, block: &Block, output: &str, format_type: Option<String>) -> Block {
        let mut results_block = Block::new("results", None, output);
        
        // Add "for" modifier pointing to the original block
        if let Some(block_name) = &block.name {
            results_block.add_modifier("for", block_name);
        }
        
        // Set format if specified or determine automatically
        let format = format_type.unwrap_or_else(|| self.determine_format_from_content(output));
        results_block.add_modifier("format", &format);
        
        // Apply default display setting
        results_block.add_modifier("display", "block");
        
        // Inherit relevant modifiers from the original block
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
    
    // Determine if a results block should be displayed inline
    pub fn should_display_inline(&self, block: &Block) -> bool {
        if let Some(display) = block.get_modifier("display") {
            display == "inline"
        } else {
            false // Default is block display
        }
    }
    
    // Determine if a results block should be displayed at all
    pub fn should_display(&self, block: &Block) -> bool {
        if let Some(display) = block.get_modifier("display") {
            display != "none"
        } else {
            true // Default is to display
        }
    }
    
    // Determine the format for a results block
    pub fn determine_format(&self, block: &Block) -> String {
        if let Some(format) = block.get_modifier("format") {
            format.to_string()
        } else {
            self.determine_format_from_content(&block.content)
        }
    }
    
    // Auto-detect content format based on its structure
    pub fn determine_format_from_content(&self, content: &str) -> String {
        let trimmed = content.trim();
        
        // Check if it's JSON
        if (trimmed.starts_with('{') && trimmed.ends_with('}')) || 
           (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            return "json".to_string();
        }
        
        // Check if it's CSV
        if trimmed.contains(',') && 
           trimmed.lines().count() > 1 && 
           trimmed.lines().all(|line| line.contains(',')) {
            return "csv".to_string();
        }
        
        // Check if it's Markdown (contains common MD markers)
        if trimmed.contains('#') || 
           trimmed.contains("```") || 
           (trimmed.contains('|') && trimmed.contains('-')) {
            return "markdown".to_string();
        }
        
        // Default to plain text
        "plain".to_string()
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
    
    // Apply modifiers to a variable value before substitution (legacy version)
    pub fn apply_block_modifiers_to_variable(&self, var_name: &str, value: &str) -> String {
        // Check if we have a block with this name to get modifiers from
        if let Some(block) = self.blocks.get(var_name) {
            let mut processed = value.to_string();
            
            // Apply trim if specified
            processed = self.apply_trim(block, &processed);
            
            // Apply max_lines if specified
            processed = self.apply_max_lines(block, &processed);
            
            return processed;
        }
        
        // If no block found or no modifiers applied, return the original value
        value.to_string()
    }
    
    /// Apply modifiers to a variable value
    pub fn apply_modifiers_to_variable(&self, var_name: &str, value: &str, modifiers: &HashMap<String, String>) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying modifiers to variable '{}' with {} modifiers", 
                     var_name, modifiers.len());
            for (k, v) in modifiers {
                println!("DEBUG:   '{}' = '{}'", k, v);
            }
        }
        
        // Start with the original value
        let mut result = value.to_string();
        
        // Apply modifiers in a specific order for predictable results
        
        // 1. Apply transformations first (uppercase, lowercase, substring, etc.)
        if let Some(transform) = modifiers.get("transform") {
            result = self.apply_transformation(&result, transform);
        }
        
        // 2. Apply regex replacements if specified
        if let Some(regex_pattern) = modifiers.get("regex") {
            if let Some(replacement) = modifiers.get("replacement") {
                result = self.apply_regex_replacement(&result, regex_pattern, replacement);
            }
        }
        
        // 3. Apply limit modifier to truncate content
        if let Some(limit_str) = modifiers.get("limit") {
            if let Ok(limit) = limit_str.parse::<usize>() {
                result = self.apply_limit(&result, limit);
            }
        }
        
        // 4. Apply format modifier (markdown, json, code, plain)
        if let Some(format) = modifiers.get("format") {
            result = self.apply_format(var_name, &result, format);
        }
        
        // 5. Apply highlighting for code blocks
        if modifiers.get("highlight").map_or(false, |v| v == "true") {
            result = self.apply_highlighting(var_name, &result);
        }
        
        // 6. Apply include modifiers (include_code, include_results)
        result = self.apply_include_modifiers(var_name, &result, modifiers);
        
        // 7. Apply conditional modifiers (include_sensitive)
        result = self.apply_conditional_modifiers(var_name, &result, modifiers);
        
        // 8. Apply JSON path extraction if specified
        if let Some(json_path) = modifiers.get("json_path") {
            result = self.extract_json_path(&result, json_path);
        }
        
        // 9. Apply trim operations
        if let Some(trim_type) = modifiers.get("trim") {
            result = self.apply_trim_modifier(&result, trim_type);
        }
        
        // 10. Apply line operations (head, tail)
        if let Some(head_str) = modifiers.get("head") {
            if let Ok(head) = head_str.parse::<usize>() {
                result = self.apply_head(&result, head);
            }
        } else if let Some(tail_str) = modifiers.get("tail") {
            if let Ok(tail) = tail_str.parse::<usize>() {
                result = self.apply_tail(&result, tail);
            }
        }
        
        // Apply any block-level modifiers if they exist and weren't explicitly overridden
        if let Some(block) = self.blocks.get(var_name) {
            // Only apply block modifiers for attributes not already specified
            for (key, value) in &block.modifiers {
                if !modifiers.contains_key(key.as_str()) {
                    match key.as_str() {
                        "trim" => {
                            result = self.apply_trim(block, &result);
                        },
                        "max_lines" => {
                            result = self.apply_max_lines(block, &result);
                        },
                        _ => {}
                    }
                }
            }
        }
        
        if debug_enabled {
            println!("DEBUG: After applying modifiers, result length: {}", result.len());
        }
        
        result
    }
    
    /// Apply format modifier (markdown, json, code, plain)
    fn apply_format(&self, var_name: &str, content: &str, format_type: &str) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying format '{}' to variable '{}'", format_type, var_name);
        }
        
        match format_type {
            "markdown" => {
                // Convert to markdown format
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
                    let mut markdown = String::new();
                    
                    if let Some(obj) = json_value.as_object() {
                        markdown.push_str("## Data\n\n");
                        for (key, value) in obj {
                            markdown.push_str(&format!("**{}**: ", key));
                            
                            match value {
                                serde_json::Value::Array(arr) => {
                                    markdown.push_str("\n");
                                    for item in arr {
                                        markdown.push_str(&format!("- {}\n", item));
                                    }
                                },
                                _ => {
                                    markdown.push_str(&format!("{}\n", value));
                                }
                            }
                        }
                    } else if let Some(arr) = json_value.as_array() {
                        markdown.push_str("## Items\n\n");
                        for item in arr {
                            markdown.push_str(&format!("- {}\n", item));
                        }
                    }
                    
                    return markdown;
                }
                
                // If not JSON, return as-is
                content.to_string()
            },
            "json" => {
                // Try to pretty-print JSON if it's valid
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
                        return pretty;
                    }
                }
                
                // If not valid JSON, return as-is
                content.to_string()
            },
            "code" => {
                // Format as code block
                format!("```\n{}\n```", content)
            },
            "plain" => {
                // Return as plain text
                content.to_string()
            },
            _ => content.to_string()
        }
    }
    
    /// Apply limit modifier to truncate content
    fn apply_limit(&self, content: &str, limit: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() <= limit {
            return content.to_string();
        }
        
        // Take only the first 'limit' lines
        let limited = lines.iter().take(limit).cloned().collect::<Vec<&str>>().join("\n");
        format!("{}\n...(truncated, showing {} of {} lines)", limited, limit, lines.len())
    }
    
    /// Apply transformation modifiers (uppercase, lowercase, substring, etc.)
    fn apply_transformation(&self, content: &str, transform: &str) -> String {
        match transform {
            "uppercase" => content.to_uppercase(),
            "lowercase" => content.to_lowercase(),
            "capitalize" => {
                let mut result = String::new();
                let mut capitalize_next = true;
                
                for c in content.chars() {
                    if c.is_alphabetic() {
                        if capitalize_next {
                            result.extend(c.to_uppercase());
                            capitalize_next = false;
                        } else {
                            result.push(c);
                        }
                    } else {
                        result.push(c);
                        if c.is_whitespace() || c == '.' || c == '!' || c == '?' {
                            capitalize_next = true;
                        }
                    }
                }
                result
            },
            "trim" => content.trim().to_string(),
            transform if transform.starts_with("substring(") && transform.ends_with(")") => {
                // Parse substring parameters
                let params = transform.trim_start_matches("substring(").trim_end_matches(")");
                let parts: Vec<&str> = params.split(',').collect();
                
                if parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<usize>()) {
                        if start < content.len() {
                            let end = std::cmp::min(end, content.len());
                            return content[start..end].to_string();
                        }
                    }
                }
                
                // If parsing failed or parameters are invalid, return original content
                content.to_string()
            },
            transform if transform.starts_with("replace(") && transform.ends_with(")") => {
                // Parse replace parameters: replace(old,new)
                let params = transform.trim_start_matches("replace(").trim_end_matches(")");
                
                // Split by comma, but respect nested parentheses
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut nesting = 0;
                let mut in_quotes = false;
                
                for c in params.chars() {
                    match c {
                        '(' => {
                            nesting += 1;
                            current.push(c);
                        },
                        ')' => {
                            nesting -= 1;
                            current.push(c);
                        },
                        '"' => {
                            in_quotes = !in_quotes;
                            current.push(c);
                        },
                        ',' if nesting == 0 && !in_quotes => {
                            parts.push(current);
                            current = String::new();
                        },
                        _ => current.push(c)
                    }
                }
                
                if !current.is_empty() {
                    parts.push(current);
                }
                
                if parts.len() == 2 {
                    let old = parts[0].trim();
                    let new = parts[1].trim();
                    
                    // Remove quotes if present
                    let old = if (old.starts_with('"') && old.ends_with('"')) || 
                               (old.starts_with('\'') && old.ends_with('\'')) {
                        &old[1..old.len()-1]
                    } else {
                        old
                    };
                    
                    let new = if (new.starts_with('"') && new.ends_with('"')) || 
                               (new.starts_with('\'') && new.ends_with('\'')) {
                        &new[1..new.len()-1]
                    } else {
                        new
                    };
                    
                    return content.replace(old, new);
                }
                
                content.to_string()
            },
            _ => content.to_string()
        }
    }
    
    /// Apply regex replacement
    fn apply_regex_replacement(&self, content: &str, pattern: &str, replacement: &str) -> String {
        match regex::Regex::new(pattern) {
            Ok(re) => re.replace_all(content, replacement).to_string(),
            Err(_) => {
                println!("WARNING: Invalid regex pattern: {}", pattern);
                content.to_string()
            }
        }
    }
    
    /// Apply trim modifier with different options
    fn apply_trim_modifier(&self, content: &str, trim_type: &str) -> String {
        match trim_type {
            "true" | "yes" | "1" | "both" => content.trim().to_string(),
            "start" | "left" => content.trim_start().to_string(),
            "end" | "right" => content.trim_end().to_string(),
            "lines" => {
                // Trim each line individually
                content.lines()
                    .map(|line| line.trim())
                    .collect::<Vec<&str>>()
                    .join("\n")
            },
            "empty_lines" => {
                // Remove empty lines
                content.lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect::<Vec<&str>>()
                    .join("\n")
            },
            _ => content.to_string()
        }
    }
    
    /// Extract data using JSON path
    fn extract_json_path(&self, content: &str, json_path: &str) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Extracting JSON path '{}' from content", json_path);
        }
        
        // Try to parse the content as JSON
        match serde_json::from_str::<serde_json::Value>(content) {
            Ok(json_value) => {
                // Split the path by dots and navigate the JSON structure
                let path_parts: Vec<&str> = json_path.split('.').collect();
                let mut current = &json_value;
                
                for part in path_parts {
                    if debug_enabled {
                        println!("DEBUG: Processing path part: '{}'", part);
                    }
                    
                    // Handle array indexing
                    if part.ends_with(']') && part.contains('[') {
                        let bracket_pos = part.find('[').unwrap();
                        let object_key = &part[..bracket_pos];
                        let index_str = &part[bracket_pos+1..part.len()-1];
                        
                        if debug_enabled {
                            println!("DEBUG: Array indexing - key: '{}', index: '{}'", object_key, index_str);
                        }
                        
                        // Navigate to the object first
                        if !object_key.is_empty() {
                            if let Some(obj) = current.get(object_key) {
                                current = obj;
                            } else {
                                if debug_enabled {
                                    println!("DEBUG: Key '{}' not found", object_key);
                                }
                                return format!("JSON path error: key '{}' not found", object_key);
                            }
                        }
                        
                        // Then get the array element
                        if let Ok(index) = index_str.parse::<usize>() {
                            if let Some(array) = current.as_array() {
                                if index < array.len() {
                                    current = &array[index];
                                } else {
                                    if debug_enabled {
                                        println!("DEBUG: Index {} out of bounds", index);
                                    }
                                    return format!("JSON path error: index {} out of bounds", index);
                                }
                            } else {
                                if debug_enabled {
                                    println!("DEBUG: Not an array");
                                }
                                return "JSON path error: not an array".to_string();
                            }
                        } else {
                            if debug_enabled {
                                println!("DEBUG: Invalid array index: '{}'", index_str);
                            }
                            return format!("JSON path error: invalid array index '{}'", index_str);
                        }
                    } else {
                        // Regular object property
                        if let Some(obj) = current.get(part) {
                            current = obj;
                        } else {
                            if debug_enabled {
                                println!("DEBUG: Key '{}' not found", part);
                            }
                            return format!("JSON path error: key '{}' not found", part);
                        }
                    }
                }
                
                // Convert the final value to string
                let result = match current {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "null".to_string(),
                    _ => serde_json::to_string_pretty(current).unwrap_or_else(|_| current.to_string()),
                };
                
                if debug_enabled {
                    println!("DEBUG: Extracted value: '{}'", result);
                }
                
                result
            },
            Err(e) => {
                if debug_enabled {
                    println!("DEBUG: Failed to parse JSON: {}", e);
                }
                format!("JSON path error: content is not valid JSON")
            }
        }
    }
    
    /// Apply head operation (take first N lines)
    fn apply_head(&self, content: &str, n: usize) -> String {
        content.lines()
            .take(n)
            .collect::<Vec<&str>>()
            .join("\n")
    }
    
    /// Apply tail operation (take last N lines)
    fn apply_tail(&self, content: &str, n: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > n { lines.len() - n } else { 0 };
        
        lines[start..]
            .iter()
            .cloned()
            .collect::<Vec<&str>>()
            .join("\n")
    }
    
    /// Apply highlighting modifier for code blocks
    fn apply_highlighting(&self, var_name: &str, content: &str) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying highlighting to variable '{}'", var_name);
        }
        
        // First check if this is a reference to a code block
        if let Some(block) = self.blocks.get(var_name) {
            if block.block_type.starts_with("code:") {
                // Extract language from block type (e.g., "code:python" -> "python")
                let language = block.block_type.split(':').nth(1).unwrap_or("text");
                if debug_enabled {
                    println!("DEBUG: Detected language '{}' from block type", language);
                }
                return format!("```{}\n{}\n```", language, content);
            } else if block.block_type == "shell" {
                // Shell blocks get bash highlighting
                if debug_enabled {
                    println!("DEBUG: Detected shell block, using bash highlighting");
                }
                return format!("```bash\n{}\n```", content);
            }
        }
        
        // If not a recognized code block, try to auto-detect language
        let trimmed = content.trim();
        
        // Auto-detect Python
        if trimmed.contains("def ") || trimmed.contains("import ") || 
           trimmed.contains("class ") || trimmed.starts_with("#!/usr/bin/env python") {
            if debug_enabled {
                println!("DEBUG: Auto-detected Python code");
            }
            return format!("```python\n{}\n```", content);
        }
        
        // Auto-detect JavaScript
        if trimmed.contains("function ") || trimmed.contains("const ") || 
           trimmed.contains("let ") || trimmed.contains("var ") || 
           trimmed.contains("=> {") {
            if debug_enabled {
                println!("DEBUG: Auto-detected JavaScript code");
            }
            return format!("```javascript\n{}\n```", content);
        }
        
        // Auto-detect Rust
        if trimmed.contains("fn ") || trimmed.contains("impl ") || 
           trimmed.contains("struct ") || trimmed.contains("enum ") || 
           trimmed.contains("use std::") {
            if debug_enabled {
                println!("DEBUG: Auto-detected Rust code");
            }
            return format!("```rust\n{}\n```", content);
        }
        
        // Auto-detect JSON
        if (trimmed.starts_with('{') && trimmed.ends_with('}')) || 
           (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            if debug_enabled {
                println!("DEBUG: Auto-detected JSON data");
            }
            return format!("```json\n{}\n```", content);
        }
        
        // Auto-detect shell commands
        if trimmed.contains("#!/bin/bash") || trimmed.contains("#!/bin/sh") || 
           trimmed.starts_with("$ ") || trimmed.contains(" | grep") || 
           trimmed.contains("sudo ") {
            if debug_enabled {
                println!("DEBUG: Auto-detected shell commands");
            }
            return format!("```bash\n{}\n```", content);
        }
        
        // If language detection failed, use plain code block
        if debug_enabled {
            println!("DEBUG: No language detected, using plain code block");
        }
        format!("```\n{}\n```", content)
    }
    
    /// Apply include modifiers (include_code, include_results)
    fn apply_include_modifiers(&self, var_name: &str, content: &str, modifiers: &HashMap<String, String>) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying include modifiers to variable '{}'", var_name);
        }
        
        let mut result = content.to_string();
        let mut sections = Vec::new();
        
        // Handle include_code modifier
        if modifiers.get("include_code").map_or(false, |v| v == "true") {
            if debug_enabled {
                println!("DEBUG: Including original code");
            }
            
            // Get the original code content
            if let Some(block) = self.blocks.get(var_name) {
                let code_content = block.content.clone();
                
                // Determine language for syntax highlighting
                let language = if block.block_type.starts_with("code:") {
                    block.block_type.split(':').nth(1).unwrap_or("text")
                } else if block.block_type == "shell" {
                    "bash"
                } else {
                    "text"
                };
                
                // Format the code with proper syntax highlighting
                let formatted_code = format!("```{}\n{}\n```", language, code_content);
                sections.push(("Code", formatted_code));
            }
        }
        
        // Handle include_results modifier
        if modifiers.get("include_results").map_or(false, |v| v == "true") {
            if debug_enabled {
                println!("DEBUG: Including execution results");
            }
            
            // Look for results in different formats
            let results_key = format!("{}_results", var_name);
            let dot_results_key = format!("{}.results", var_name);
            
            if let Some(results) = self.outputs.get(&results_key).or_else(|| self.outputs.get(&dot_results_key)) {
                // Check if we should format the results
                let formatted_results = if modifiers.get("format_results").map_or(false, |v| v == "true") {
                    // Try to determine the format of the results
                    if results.trim().starts_with('{') || results.trim().starts_with('[') {
                        // Looks like JSON
                        format!("```json\n{}\n```", results)
                    } else if results.contains("def ") || results.contains("class ") {
                        // Looks like Python
                        format!("```python\n{}\n```", results)
                    } else {
                        // Default formatting
                        format!("```\n{}\n```", results)
                    }
                } else {
                    // No formatting requested
                    results.clone()
                };
                
                sections.push(("Results", formatted_results));
            } else if debug_enabled {
                println!("DEBUG: No results found for '{}'", var_name);
            }
        }
        
        // Handle include_error modifier
        if modifiers.get("include_error").map_or(false, |v| v == "true") {
            if debug_enabled {
                println!("DEBUG: Including error information if available");
            }
            
            // Look for error in outputs
            let error_key = format!("{}_error", var_name);
            
            if let Some(error) = self.outputs.get(&error_key) {
                // Format the error message
                let formatted_error = format!("```\n{}\n```", error);
                sections.push(("Error", formatted_error));
            }
        }
        
        // If we have sections to add and the original content isn't empty
        if !sections.is_empty() {
            // Start with the original content if it's not empty
            let mut combined = if !content.trim().is_empty() {
                // Check if we should put the original content first or last
                if modifiers.get("content_first").map_or(true, |v| v == "true") {
                    // Original content first (default)
                    result
                } else {
                    // Original content will be added last
                    String::new()
                }
            } else {
                // Empty original content
                String::new()
            };
            
            // Add each section
            for (title, section_content) in sections {
                if !combined.is_empty() {
                    combined.push_str("\n\n");
                }
                combined.push_str(&format!("### {}\n\n{}", title, section_content));
            }
            
            // Add original content at the end if requested
            if !content.trim().is_empty() && modifiers.get("content_first").map_or(false, |v| v == "false") {
                combined.push_str("\n\n### Output\n\n");
                combined.push_str(&content);
            }
            
            return combined;
        }
        
        // If no sections were added, return the original content
        result
    }
    
    /// Apply conditional modifiers (include_sensitive, if, unless)
    fn apply_conditional_modifiers(&self, var_name: &str, content: &str, modifiers: &HashMap<String, String>) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying conditional modifiers to variable '{}'", var_name);
        }
        
        let mut result = content.to_string();
        
        // Handle 'if' modifier - only include content if condition is true
        if let Some(condition_var) = modifiers.get("if") {
            if debug_enabled {
                println!("DEBUG: Found 'if' condition: {}", condition_var);
            }
            
            let condition_met = self.evaluate_condition(condition_var);
            
            if debug_enabled {
                println!("DEBUG: Condition '{}' evaluated to: {}", condition_var, condition_met);
            }
            
            if !condition_met {
                // Condition not met, return empty string
                if debug_enabled {
                    println!("DEBUG: Condition not met, returning empty string");
                }
                return String::new();
            }
        }
        
        // Handle 'unless' modifier - only include content if condition is false
        if let Some(condition_var) = modifiers.get("unless") {
            if debug_enabled {
                println!("DEBUG: Found 'unless' condition: {}", condition_var);
            }
            
            let condition_met = self.evaluate_condition(condition_var);
            
            if debug_enabled {
                println!("DEBUG: Condition '{}' evaluated to: {}", condition_var, condition_met);
            }
            
            if condition_met {
                // Condition met, return empty string (since this is 'unless')
                if debug_enabled {
                    println!("DEBUG: 'Unless' condition met, returning empty string");
                }
                return String::new();
            }
        }
        
        // Handle include_sensitive modifier
        if let Some(condition_var) = modifiers.get("include_sensitive") {
            if debug_enabled {
                println!("DEBUG: Found include_sensitive condition: {}", condition_var);
            }
            
            let include_sensitive = self.evaluate_condition(condition_var);
            
            if debug_enabled {
                println!("DEBUG: include_sensitive evaluated to: {}", include_sensitive);
            }
            
            if !include_sensitive {
                // Don't include sensitive information
                if debug_enabled {
                    println!("DEBUG: Removing sensitive information from content");
                }
                
                // Check if content is JSON
                if let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(&result) {
                    if let Some(obj) = json_value.as_object_mut() {
                        // Define sensitive field patterns
                        let sensitive_fields = [
                            "password", "passwd", "secret", "token", "api_key", "apikey", 
                            "private_key", "privatekey", "sensitive", "credential", 
                            "auth", "authentication", "key", "cert", "certificate"
                        ];
                        
                        // Remove all fields that match sensitive patterns
                        let keys_to_remove: Vec<String> = obj.keys()
                            .filter(|k| {
                                let k_lower = k.to_lowercase();
                                sensitive_fields.iter().any(|&pattern| k_lower.contains(pattern))
                            })
                            .cloned()
                            .collect();
                        
                        for key in keys_to_remove {
                            if debug_enabled {
                                println!("DEBUG: Removing sensitive field: {}", key);
                            }
                            obj.remove(&key);
                        }
                        
                        // Also recursively check nested objects
                        self.redact_sensitive_fields(obj);
                        
                        if let Ok(filtered) = serde_json::to_string_pretty(&json_value) {
                            result = filtered;
                        }
                    }
                } else if debug_enabled {
                    println!("DEBUG: Content is not JSON, applying text-based redaction");
                    
                    // For non-JSON content, try to redact common patterns
                    let patterns = [
                        ("password\\s*[:=]\\s*['\\\"].*?['\\\"]", "password: \"[REDACTED]\""),
                        ("api[_-]?key\\s*[:=]\\s*['\\\"].*?['\\\"]", "api_key: \"[REDACTED]\""),
                        ("secret\\s*[:=]\\s*['\\\"].*?['\\\"]", "secret: \"[REDACTED]\""),
                        ("token\\s*[:=]\\s*['\\\"].*?['\\\"]", "token: \"[REDACTED]\""),
                    ];
                    
                    for (pattern, replacement) in &patterns {
                        if let Ok(re) = regex::Regex::new(pattern) {
                            let replacement_str = replacement.to_string();
                            result = re.replace_all(&result, |_: &regex::Captures| replacement_str.clone()).to_string();
                        }
                    }
                }
            }
        }
        
        // Handle conditional sections with ${if:condition}...${endif} syntax
        result = self.process_conditional_blocks(&result);
        
        result
    }
    
    /// Helper method to evaluate a condition string
    fn evaluate_condition(&self, condition: &str) -> bool {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Evaluating condition: {}", condition);
        }
        
        // Handle direct boolean values
        if condition == "true" || condition == "1" || condition == "yes" || condition == "on" {
            return true;
        }
        if condition == "false" || condition == "0" || condition == "no" || condition == "off" {
            return false;
        }
        
        // Handle negation with ! prefix
        if condition.starts_with('!') {
            return !self.evaluate_condition(&condition[1..]);
        }
        
        // Handle comparison operations
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim();
                
                // Resolve variables in both sides
                let left_value = if left.starts_with("${") && left.ends_with("}") {
                    let var_name = &left[2..left.len()-1];
                    self.lookup_variable(var_name).unwrap_or_default()
                } else {
                    left.to_string()
                };
                
                let right_value = if right.starts_with("${") && right.ends_with("}") {
                    let var_name = &right[2..right.len()-1];
                    self.lookup_variable(var_name).unwrap_or_default()
                } else {
                    right.to_string()
                };
                
                if debug_enabled {
                    println!("DEBUG: Comparing '{}' == '{}'", left_value, right_value);
                }
                
                return left_value == right_value;
            }
        }
        
        // Handle not equals comparison
        if condition.contains("!=") {
            let parts: Vec<&str> = condition.split("!=").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim();
                
                // Resolve variables in both sides
                let left_value = if left.starts_with("${") && left.ends_with("}") {
                    let var_name = &left[2..left.len()-1];
                    self.lookup_variable(var_name).unwrap_or_default()
                } else {
                    left.to_string()
                };
                
                let right_value = if right.starts_with("${") && right.ends_with("}") {
                    let var_name = &right[2..right.len()-1];
                    self.lookup_variable(var_name).unwrap_or_default()
                } else {
                    right.to_string()
                };
                
                if debug_enabled {
                    println!("DEBUG: Comparing '{}' != '{}'", left_value, right_value);
                }
                
                return left_value != right_value;
            }
        }
        
        // Handle variable existence check
        if condition.starts_with("exists:") {
            let var_name = &condition[7..];
            return self.lookup_variable(var_name).is_some();
        }
        
        // Handle empty check
        if condition.starts_with("empty:") {
            let var_name = &condition[6..];
            return self.lookup_variable(var_name).map_or(true, |v| v.trim().is_empty());
        }
        
        // Handle not-empty check
        if condition.starts_with("not-empty:") {
            let var_name = &condition[10..];
            return self.lookup_variable(var_name).map_or(false, |v| !v.trim().is_empty());
        }
        
        // Default: treat as variable name and check if it's truthy
        if let Some(value) = self.lookup_variable(condition) {
            let value_lower = value.to_lowercase();
            return value_lower == "true" || value_lower == "1" || 
                   value_lower == "yes" || value_lower == "on" || 
                   !value.is_empty();
        }
        
        // If all else fails, return false
        false
    }
    
    /// Recursively redact sensitive fields in nested JSON objects
    fn redact_sensitive_fields(&self, obj: &mut serde_json::Map<String, serde_json::Value>) {
        let sensitive_fields = [
            "password", "passwd", "secret", "token", "api_key", "apikey", 
            "private_key", "privatekey", "sensitive", "credential", 
            "auth", "authentication", "key", "cert", "certificate"
        ];
        
        // Process all object values
        for (_, value) in obj.iter_mut() {
            if let serde_json::Value::Object(nested_obj) = value {
                // Check and redact fields in this nested object
                let keys_to_redact: Vec<String> = nested_obj.keys()
                    .filter(|k| {
                        let k_lower = k.to_lowercase();
                        sensitive_fields.iter().any(|&pattern| k_lower.contains(pattern))
                    })
                    .cloned()
                    .collect();
                
                for key in keys_to_redact {
                    nested_obj.insert(key, serde_json::Value::String("[REDACTED]".to_string()));
                }
                
                // Recursively process deeper nested objects
                self.redact_sensitive_fields(nested_obj);
            } else if let serde_json::Value::Array(arr) = value {
                // Process arrays of objects
                for item in arr.iter_mut() {
                    if let serde_json::Value::Object(nested_obj) = item {
                        self.redact_sensitive_fields(nested_obj);
                    }
                }
            }
        }
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
    
    // Update the execution method to automatically create results blocks
    pub fn execute_block_with_results(&mut self, name: &str) -> Result<Block, ExecutorError> {
        match self.execute_block(name) {
            Ok(output) => {
                // Get the original block
                let block = self.blocks.get(name).unwrap().clone();
                
                // Determine format from block modifiers or content
                let format = block.get_modifier("format")
                    .map(|f| f.to_string())
                    .or_else(|| Some(self.determine_format_from_content(&output)));
                
                // Create a results block
                let results_block = self.generate_results_block(&block, &output, format.clone());
                
                // Process the results content with all modifiers
                let processed_content = self.process_results_content(&results_block, &output);
                
                // Create the final results block with processed content
                let final_block = self.generate_results_block(&block, &processed_content, format);
                
                // Store the results in outputs if the block has a name
                if let Some(block_name) = &block.name {
                    let results_name = format!("{}_results", block_name);
                    self.outputs.insert(results_name, processed_content);
                }
                
                Ok(final_block)
            },
            Err(err) => {
                // Get the original block
                if let Some(block) = self.blocks.get(name) {
                    // Create an error results block
                    let _error_block = self.generate_error_results_block(block, &err.to_string());
                    
                    // Store the error in outputs if the block has a name
                    if let Some(block_name) = &block.name {
                        let error_name = format!("{}_error", block_name);
                        self.outputs.insert(error_name, err.to_string());
                    }
                    
                    // If there's a fallback, try to execute it
                    let fallback_to_execute = if let Some(fallback_name) = self.fallbacks.get(name) {
                        println!("Trying fallback block: {}", fallback_name);
                        Some(fallback_name.clone())
                    } else {
                        None
                    };
                    
                    // Execute the fallback if we found one
                    if let Some(fallback_name) = fallback_to_execute {
                        return self.execute_block_with_results(&fallback_name);
                    }
                }
                
                // Return the error anyway so the caller knows execution failed
                Err(err)
            }
        }
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
        
        // Debug: Print all outputs
        println!("DEBUG: All outputs:");
        for (k, v) in &self.outputs {
            println!("DEBUG:   '{}' => '{}' (length: {})", k, 
                     if v.len() > 30 { &v[..30] } else { v }, v.len());
        }
        
        // Debug: Print the current state of outputs after processing
        self.debug_print_outputs("AFTER PROCESSING");
        
        let mut updated_doc = self.current_document.clone();
        
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
}
// Implement Default for MetaLanguageExecutor
impl Default for MetaLanguageExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
