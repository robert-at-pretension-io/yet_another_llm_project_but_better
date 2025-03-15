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
}

impl MetaLanguageExecutor {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            outputs: HashMap::new(),
            fallbacks: HashMap::new(),
            cache: HashMap::new(),
            current_document: String::new(),
            processing_blocks: Vec::new(),
        }
    }

    // Process a document
    pub fn process_document(&mut self, content: &str) -> Result<(), ExecutorError> {
        // Parse the document
        let blocks = parse_document(content).map_err(|e| ExecutorError::ExecutionFailed(e.to_string()))?;
        
        // Clear existing state (keeping cache)
        self.blocks.clear();
        self.outputs.clear();
        self.fallbacks.clear();
        self.current_document = content.to_string();
        self.processing_blocks.clear();
        
        // Register all blocks and identify fallbacks
        for block in &blocks {
            if let Some(name) = &block.name {
                self.blocks.insert(name.clone(), block.clone());
                
                // Check if this is a fallback block
                if name.ends_with("-fallback") {
                    let original_name = name.trim_end_matches("-fallback");
                    self.fallbacks.insert(original_name.to_string(), name.clone());
                }
                
                // Store content of data blocks directly in outputs
                if block.block_type == "data" {
                    self.outputs.insert(name.clone(), block.content.clone());
                }
            }
        }
        
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
                    let modified_content = self.apply_modifiers_to_variable(&name, &processed_content);
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
            if let Some(name) = &block.name {
                if self.is_executable_block(&block) && !self.has_explicit_dependency(&block) {
                    self.execute_block(name)?;
                }
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
        // Check for circular dependencies
        if self.processing_blocks.contains(&name.to_string()) {
            return Err(ExecutorError::CircularDependency(name.to_string()));
        }
        
        // Check if block exists
        let block = match self.blocks.get(name) {
            Some(b) => b.clone(),
            None => return Err(ExecutorError::BlockNotFound(name.to_string())),
        };
        
        // Check if result is cached
        if self.is_cacheable(&block) {
            if let Some((result, timestamp)) = self.cache.get(name) {
                // Check if cache is still valid (e.g., within timeout)
                let now = Instant::now();
                let timeout = self.get_timeout(&block);
                
                if now.duration_since(*timestamp) < timeout {
                    return Ok(result.clone());
                }
            }
        }
        
        // Mark block as being processed (for dependency tracking)
        self.processing_blocks.push(name.to_string());
        
        // Execute dependencies first
        for (key, value) in &block.modifiers {
            if key == "depends" || key == "requires" {
                self.execute_block(value)?;
            }
        }
        
        // Process variable references in content
        // We need to get the latest content from the blocks map, as it might have been updated
        let block_content = if let Some(updated_block) = self.blocks.get(name) {
            updated_block.content.clone()
        } else {
            block.content.clone()
        };
        
        let processed_content = self.process_variable_references(&block_content);
        
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
                // Store output with the block name
                self.outputs.insert(name.to_string(), output.clone());
                
                // Also store with block_name.results format
                let results_key = format!("{}.results", name);
                self.outputs.insert(results_key, output.clone());
                
                // Cache if needed
                if self.is_cacheable(&block) {
                    self.cache.insert(name.to_string(), (output.clone(), Instant::now()));
                }
                Ok(output)
            },
            Err(e) => {
                // Use fallback
                if let Some(fallback_name) = self.fallbacks.get(name) {
                    println!("Block '{}' failed, using fallback: {}", name, fallback_name);
                    let fallback_name_clone = fallback_name.clone();
                    self.execute_block(&fallback_name_clone)
                } else {
                    Err(e)
                }
            }
        }
    }
    
    // Check if a block's result should be cached
    pub fn is_cacheable(&self, block: &Block) -> bool {
        block.modifiers.iter().any(|(key, value)| key == "cache_result" && value == "true")
    }
    
    // Get timeout duration for a block
    pub fn get_timeout(&self, block: &Block) -> Duration {
        for (key, value) in &block.modifiers {
            if key == "timeout" {
                if let Ok(seconds) = value.parse::<u64>() {
                    return Duration::from_secs(seconds);
                }
            }
        }
        // Default timeout (10 minutes)
        Duration::from_secs(600)
    }
    
    // Process variable references like ${block_name} or ${block_name:fallback_value}
    pub fn process_variable_references(&self, content: &str) -> String {
        self.process_variable_references_internal(content, &mut Vec::new())
    }
    
    // Helper function to look up a variable value, handling dotted names
    fn lookup_variable(&self, var_name: &str) -> Option<String> {
        println!("lookup_variable called with: '{}'", var_name);
        
        // First try direct lookup
        if let Some(value) = self.outputs.get(var_name) {
            println!("  Direct lookup succeeded for '{}'", var_name);
            return Some(value.clone());
        }
        
        // If the name contains dots, it might be a reference to a result
        // Format could be: block_name.results
        if var_name.contains('.') {
            println!("  Variable contains dots: '{}'", var_name);
            let parts: Vec<&str> = var_name.split('.').collect();
            println!("  Split into parts: {:?}", parts);
            
            if parts.len() == 2 {
                let block_name = parts[0];
                let suffix = parts[1];
                
                println!("  Checking block_name: '{}', suffix: '{}'", block_name, suffix);
                
                // Handle common suffixes
                if suffix == "results" {
                    let results_key = format!("{}_results", block_name);
                    println!("  Looking up results_key: '{}'", results_key);
                    
                    if let Some(value) = self.outputs.get(&results_key) {
                        println!("  Found value for '{}': '{}'", results_key, value);
                        return Some(value.clone());
                    }
                } else if suffix == "error" {
                    let error_key = format!("{}_error", block_name);
                    println!("  Looking up error_key: '{}'", error_key);
                    
                    if let Some(value) = self.outputs.get(&error_key) {
                        println!("  Found value for '{}': '{}'", error_key, value);
                        return Some(value.clone());
                    }
                }
            }
        }
        
        println!("  No value found for '{}'", var_name);
        None
    }
    
    // Internal implementation that tracks processing variables to detect circular references
    fn process_variable_references_internal(&self, content: &str, processing_vars: &mut Vec<String>) -> String {
        println!("process_variable_references_internal called with: '{}'", content);
        println!("Current processing_vars: {:?}", processing_vars);
        
        let mut result = content.to_string();
        
        // Find all variable references
        let references = extract_variable_references(content);
        
        // Replace each reference with its value
        for var_name in references {
            // Check for circular references
            if processing_vars.contains(&var_name) {
                println!("Warning: Circular reference detected for variable: {}", var_name);
                continue;
            }
            
            // The original variable reference to be replaced if a value is found
            let var_ref = format!("${{{}}}", var_name);
            
            // Check if the variable name contains a fallback value (format: var_name:fallback)
            let (actual_var_name, inline_fallback) = if var_name.contains(':') {
                let parts: Vec<&str> = var_name.splitn(2, ':').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (var_name.clone(), None)
            };
            
            // Debug output for troubleshooting
            println!("Looking for variable: {}", actual_var_name);
            
            // Debug: Print all available outputs for troubleshooting
            println!("Available outputs:");
            for (k, v) in &self.outputs {
                println!("  '{}' => '{}'", k, v);
            }
            
            // Try to get the value using our lookup function
            if let Some(value) = self.lookup_variable(&actual_var_name) {
                println!("Found value for {}: {}", actual_var_name, value);
                
                // Apply any modifiers to the value
                let modified_value = self.apply_modifiers_to_variable(&actual_var_name, &value);
                
                // Check if the value itself contains variable references
                if modified_value.contains("${") {
                    // Add this variable to the processing list to detect circular references
                    processing_vars.push(actual_var_name.clone());
                    
                    // Recursively process nested references
                    let processed_value = self.process_variable_references_internal(&modified_value, processing_vars);
                    result = result.replace(&var_ref, &processed_value);
                    
                    // Remove from processing list
                    processing_vars.retain(|v| v != &actual_var_name);
                } else {
                    // Simple replacement
                    result = result.replace(&var_ref, &modified_value);
                }
            } else if let Some(fallback_name) = self.fallbacks.get(&actual_var_name) {
                // Try registered fallback if available
                if let Some(fallback_output) = self.outputs.get(fallback_name) {
                    let value = fallback_output.clone();
                    
                    // Process nested references in fallback value
                    if value.contains("${") {
                        processing_vars.push(actual_var_name.clone());
                        let processed_value = self.process_variable_references_internal(&value, processing_vars);
                        result = result.replace(&var_ref, &processed_value);
                        processing_vars.retain(|v| v != &actual_var_name);
                    } else {
                        result = result.replace(&var_ref, &value);
                    }
                } else if let Some(fallback_value) = inline_fallback {
                    // Use inline fallback if provided
                    result = result.replace(&var_ref, &fallback_value);
                }
                // If no fallback value is available, leave the reference as is
            } else if let Some(fallback_value) = inline_fallback {
                // Use inline fallback if provided
                result = result.replace(&var_ref, &fallback_value);
            } else {
                // Check if the block has a fallback modifier
                if let Some(block) = self.blocks.get(&actual_var_name) {
                    if let Some(fallback_value) = block.get_modifier("fallback") {
                        result = result.replace(&var_ref, fallback_value);
                        continue;
                    }
                }
                // If no value or fallback found, leave the reference as is (do nothing)
                println!("No value found for variable: {}", actual_var_name);
            }
        }
        
        result
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
        
        // Check if we're in test mode
        if block.is_modifier_true("test_mode") {
            println!("DEBUG: Using test mode response");
            return Ok("This is a simulated response for testing purposes.".to_string());
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
                
                // Create a response block if the question block has a name
                if let Some(name) = &block.name {
                    let response_block_name = format!("{}_response", name);
                    println!("DEBUG: Creating response block: {}", response_block_name);
                    
                    let response_str = response.as_str();
                    let mut response_block = Block::new("response", Some(&response_block_name), response_str);
                    println!("DEBUG: Created response block with content length: {}", response_str.len());
                    
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
                    
                    // Debug: Print all outputs after adding this one
                    println!("DEBUG: Current outputs after adding response:");
                    for (k, v) in &self.outputs {
                        println!("DEBUG:   '{}' => '{}' (length: {})", k, 
                                 if v.len() > 30 { &v[..30] } else { v }, v.len());
                    }
                } else {
                    println!("DEBUG: Question block has no name, not creating a response block");
                }
                
                Ok(response)
            },
            Err(e) => {
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
    
    // Apply modifiers to a variable value before substitution
    pub fn apply_modifiers_to_variable(&self, var_name: &str, value: &str) -> String {
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
        
        // Debug: Print all outputs
        println!("DEBUG: All outputs:");
        for (k, v) in &self.outputs {
            println!("DEBUG:   '{}' => '{}' (length: {})", k, 
                     if v.len() > 30 { &v[..30] } else { v }, v.len());
        }
        
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
        }
        
        // Check if the document already contains response blocks
        let has_response_block = updated_doc.contains("[response]") || updated_doc.contains("[/response]");
        println!("DEBUG: Document already contains response blocks: {}", has_response_block);
        
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
            
            // Check for question block start with name attribute
            if trimmed.starts_with("[question") {
                in_question_block = true;
                
                // Try to extract name from the opening tag
                if let Some(name_start) = trimmed.find("name:") {
                    let name_start = name_start + 5; // skip "name:"
                    let name_end = trimmed[name_start..].find(']')
                        .map(|pos| name_start + pos)
                        .unwrap_or_else(|| trimmed[name_start..].find(' ')
                            .map(|pos| name_start + pos)
                            .unwrap_or(trimmed.len()));
                    
                    current_question_name = Some(trimmed[name_start..name_end].trim().to_string());
                    println!("DEBUG: Found question block with name: {:?}", current_question_name);
                }
            }
            
            // Check for question block end
            if trimmed == "[/question]" && in_question_block {
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
        
        while i < lines.len() {
            let line = lines[i];
            result.push_str(line);
            result.push('\n');
            
            // Check if this is the end of a question block we identified
            if let Some(pos) = question_blocks.iter().position(|(line_idx, _)| *line_idx == i) {
                let (_, question_name) = &question_blocks[pos];
                println!("DEBUG: Processing end of question block '{}' at line {}", question_name, i);
                
                // Check if the next line is already a response block
                let next_is_response = i + 1 < lines.len() && lines[i + 1].trim().starts_with("[response");
                println!("DEBUG: Next line is already a response block: {}", next_is_response);
                
                // If there's no response block following, try to add the corresponding one
                if !next_is_response {
                    // Look for a response to this specific question in the outputs
                    let response_name = format!("{}_response", question_name);
                    println!("DEBUG: Looking for response with name: '{}'", response_name);
                    
                    if let Some(output) = self.outputs.get(&response_name) {
                        println!("DEBUG: Found matching response '{}' (length: {})", response_name, output.len());
                        // Insert the response block after the question block
                        result.push_str("[response]\n");
                        result.push_str(output);
                        result.push_str("\n[/response]\n\n");
                        response_blocks_added += 1;
                        println!("DEBUG: Added response block #{} for question '{}'", response_blocks_added, question_name);
                    } else {
                        println!("DEBUG: No matching response found for question '{}'", question_name);
                    }
                }
            }
            
            i += 1;
        }
        
        println!("DEBUG: Found {} question blocks, added {} response blocks", 
                 question_blocks.len(), response_blocks_added);
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
