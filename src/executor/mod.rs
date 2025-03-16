use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::Result;
use tempfile;
use thiserror::Error;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

use crate::parser::{Block, parse_document, utils::extractors::extract_variable_references};
use crate::llm_client::{LlmClient, LlmRequestConfig, LlmProvider};
use quick_xml::events::attributes::AttrError;


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
    
    #[error("XML attribute error: {0}")]
    XmlAttributeError(#[from] AttrError),
    
    #[error("LLM API error: {0}")]
    LlmApiError(String),
    
    #[error("Missing API key: {0}")]
    MissingApiKey(String),
    
    #[error("Failed to resolve reference: {0}")]
    ReferenceResolutionFailed(String),
    
    #[error("XML parsing error: {0}")]
    XmlParsingError(#[from] quick_xml::Error)
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
        
        // Process variable references in all registered blocks using XML parsing
        let mut ops = Vec::new();
        for (name, _) in self.blocks.iter() {
            ops.push(name.clone());
        }
        
        // Now process each block's content with XML parsing and update both the block and outputs
        for name in ops {
            if let Some(block) = self.blocks.get(&name) {
                let content = block.content.clone();
                let is_executable = self.is_executable_block(block);
                
                // Process the content with XML parsing
                let processed_content = self.process_variable_references(&content)?;
                
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
    
    /// Process variable references using quick_xml
    pub fn process_variable_references(&self, content: &str) -> Result<String, ExecutorError> {
        // Check if the content might contain XML
        if !content.contains("<meta:reference") {
            return Ok(content.to_string());
        }
        
        println!("DEBUG: Processing variable references with quick_xml");
        
        let mut reader = Reader::from_str(content);
        reader.trim_text(false);
        
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"meta:reference" => {
                    // Process meta:reference tag
                    let mut target = None;
                    
                    // Extract the target attribute
                    for attr_result in e.attributes() {
                        let attr = attr_result?;
                        if attr.key.as_ref() == b"target" {
                            target = Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                    
                    if let Some(target_str) = target {
                        println!("DEBUG: Found reference tag with target: {}", target_str);
                        
                        // Look up the target in outputs
                        if let Some(value) = self.outputs.get(&target_str) {
                            println!("DEBUG: Found target '{}' in outputs, length: {}", target_str, value.len());
                            // Write the value instead of the reference tag
                            writer.write_event(Event::Text(BytesText::from_escaped(value)))?;
                        } else {
                            println!("DEBUG: Target '{}' not found in outputs", target_str);
                            // Target not found, preserve the original tag
                            writer.write_event(Event::Start(e.to_owned()))?;
                            writer.write_event(Event::End(BytesEnd::new("meta:reference")))?;
                        }
                    } else {
                        println!("DEBUG: Reference tag missing target attribute");
                        // No target attribute, preserve the original tag
                        writer.write_event(Event::Start(e.to_owned()))?;
                        writer.write_event(Event::End(BytesEnd::new("meta:reference")))?;
                    }
                },
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"meta:reference" => {
                    // Process self-closing meta:reference tag
                    let mut target = None;
                    
                    // Extract the target attribute
                    for attr_result in e.attributes() {
                        let attr = attr_result?;
                        if attr.key.as_ref() == b"target" {
                            target = Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                    
                    if let Some(target_str) = target {
                        println!("DEBUG: Found self-closing reference tag with target: {}", target_str);
                        
                        // Look up the target in outputs
                        if let Some(value) = self.outputs.get(&target_str) {
                            println!("DEBUG: Found target '{}' in outputs, length: {}", target_str, value.len());
                            // Write the value instead of the reference tag
                            writer.write_event(Event::Text(BytesText::from_escaped(value)))?;
                        } else {
                            println!("DEBUG: Target '{}' not found in outputs", target_str);
                            // Target not found, preserve the original tag
                            writer.write_event(Event::Empty(e.to_owned()))?;
                        }
                    } else {
                        println!("DEBUG: Self-closing reference tag missing target attribute");
                        // No target attribute, preserve the original tag
                        writer.write_event(Event::Empty(e.to_owned()))?;
                    }
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"meta:reference" => {
                    // Skip the end tag as we've already handled it
                    continue;
                },
                Ok(Event::Eof) => break,
                Ok(event) => writer.write_event(event)?,
                Err(e) => {
                    println!("DEBUG: XML parsing error: {}", e);
                    return Err(ExecutorError::XmlParsingError(e));
                },
            }
            
            buf.clear();
        }
        
        // Get the result as a string
        let result = writer.into_inner().into_inner();
        let result_str = String::from_utf8_lossy(&result).to_string();
        
        println!("DEBUG: Finished processing variable references, result length: {}", result_str.len());
        
        // If there are still nested references, process them recursively
        if result_str.contains("<meta:reference") {
            println!("DEBUG: Detected nested references, processing recursively");
            return self.process_variable_references(&result_str);
        }
        
        Ok(result_str)
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
    
        // Process variable references with XML parser
        let processed_content = self.process_variable_references(&block_content)?;
        
        // Execute based on block type
        let result = match block.block_type.as_str() {
            "shell" => self.execute_shell(&processed_content),
            "api" => self.execute_api(&processed_content),
            "question" => self.execute_question(&block, &processed_content),
            "code:python" => self.execute_python(&block, &processed_content),
            code if code.starts_with("code:") => {
                println!("DEBUG: Unsupported code block type '{}'. Returning processed content.", code);
                Ok(processed_content)
            }
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
    
    // Execute a shell command
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
    
    // Execute an API call
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
    
    // Execute Python code
    pub fn execute_python(&self, block: &Block, code: &str) -> Result<String, ExecutorError> {
        // Process variable references in the Python code using XML parsing
        let processed_code = self.process_variable_references(code)?;
        println!("DEBUG: Processed Python code:\n{}", processed_code);
        
        // Create a temporary Python file
        let mut tmpfile = tempfile::NamedTempFile::new().map_err(|e| ExecutorError::IoError(e))?;
        let tmp_path = tmpfile.path().to_owned();
        
        // Write the processed code to the temporary file using the file handle
        {
            let file = tmpfile.as_file_mut();
            file.write_all(processed_code.as_bytes()).map_err(|e| ExecutorError::IoError(e))?;
            file.flush().map_err(|e| ExecutorError::IoError(e))?;
        }
        
        // Execute the Python file and capture its output using python3
        println!("DEBUG: Executing Python file using python3 at {:?}", tmp_path);
        let output = Command::new("python3")
            .arg(&tmp_path)
            .output()
            .map_err(|e| ExecutorError::IoError(e))?;
        println!("DEBUG: Python execution completed with status: {:?}", output.status);
        
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
    
    fn debug_print_outputs(&self, label: &str) {
        println!("DEBUG: {}:", label);
        for (key, value) in &self.outputs {
            // Only print the first 30 chars of each value to avoid huge output
            let preview = if value.len() > 30 {
                format!("{}...", &value[..30])
            } else {
                value.clone()
            };
            println!("DEBUG:   '{}' => '{}' (length: {})", key, preview, value.len());
        }
    }
    
    fn apply_block_modifiers_to_variable(&self, name: &str, content: &str) -> String {
        // Lookup the block to get its modifiers
        if let Some(block) = self.blocks.get(name) {
            let mut modified_content = content.to_string();
            
            // Apply modifiers that affect variable content
            for (key, value) in &block.modifiers {
                match key.as_str() {
                    "trim" if value == "true" => {
                        modified_content = modified_content.trim().to_string();
                    },
                    "uppercase" if value == "true" => {
                        modified_content = modified_content.to_uppercase();
                    },
                    "lowercase" if value == "true" => {
                        modified_content = modified_content.to_lowercase();
                    },
                    "prefix" => {
                        modified_content = format!("{}{}", value, modified_content);
                    },
                    "suffix" => {
                        modified_content = format!("{}{}", modified_content, value);
                    },
                    // Add other modifiers as needed
                    _ => {}
                }
            }
            
            modified_content
        } else {
            // If block not found, return content as is
            content.to_string()
        }
    }
    
    fn generate_error_response_block(&self, original_block: &Block, error_message: &str) -> Block {
        let mut error_block = Block::new("error-response", None, error_message);
        
        // Copy the name if available
        if let Some(name) = &original_block.name {
            error_block.name = Some(format!("{}_error_response", name));
        }
        
        // Copy relevant modifiers from the original block
        for (key, value) in &original_block.modifiers {
            if matches!(key.as_str(), "format" | "display" | "max_lines" | "trim") {
                error_block.add_modifier(key, value);
            }
        }
        
        // Add reference back to the original block
        if let Some(name) = &original_block.name {
            error_block.add_modifier("for", name);
        }
        
        error_block
    }

    pub fn update_document(&self) -> Result<String, ExecutorError> {
        // Start with the original document content
        let mut updated_content = self.current_document.clone();
        
        // Keep track of replacements we've made to avoid duplicates or conflicts
        let mut replacements = HashMap::new();
        
        // Process each block that has output
        for (name, output) in &self.outputs {
            // Skip special outputs like errors or results
            if name.ends_with("_error") || name.ends_with(".results") || name.ends_with("_results") {
                continue;
            }
            
            // Only replace blocks that exist in the document
            if let Some(block) = self.blocks.get(name) {
                // Only update blocks that have changed
                if block.content != *output {
                    println!("DEBUG: Updating block '{}' in document", name);
                    
                    // Simple replacement - in a real implementation, would use a more robust XML update approach
                    // This is just a basic implementation for this code example
                    if let Some(start_tag) = updated_content.find(&format!("<meta:{}", &block.block_type)) {
                        if let Some(end_tag) = updated_content[start_tag..].find(&format!("</meta:{}>", &block.block_type)) {
                            let end_pos = start_tag + end_tag + format!("</meta:{}>", &block.block_type).len();
                            let full_tag = &updated_content[start_tag..end_pos];
                            
                            // Only replace if we haven't already replaced this exact text
                            if !replacements.contains_key(full_tag) {
                                // Create the replacement block with the same attributes but updated content
                                let mut replacement = full_tag.to_string();
                                
                                // Find the content part (after ">")
                                if let Some(content_start) = replacement.find('>') {
                                    let content_start = content_start + 1;
                                    if let Some(content_end) = replacement[content_start..].find(&format!("</meta:{}", &block.block_type)) {
                                        // Replace just the content part
                                        let prefix = &replacement[0..content_start];
                                        let suffix = &replacement[content_start+content_end..];
                                        replacement = format!("{}{}{}", prefix, output, suffix);
                                        
                                        // Store the replacement
                                        replacements.insert(full_tag.to_string(), replacement.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Apply all replacements
        for (original, replacement) in replacements {
            updated_content = updated_content.replace(&original, &replacement);
        }
        
        Ok(updated_content)
    }
}
