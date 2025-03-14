use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};


use anyhow::Result;
use tempfile;
use thiserror::Error;

use crate::parser::{Block, parse_document, extract_variable_references};

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
                "shell" | "api")
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
            if key == "depends" {
                self.execute_block(value)?;
            }
        }
        
        // Process variable references in content
        let processed_content = self.process_variable_references(&block.content);
        
        // Execute based on block type
        let result = match block.block_type.as_str() {
            "code:python" => self.execute_python(&processed_content),
            "code:javascript" => self.execute_javascript(&processed_content),
            "code:rust" => self.execute_rust(&processed_content),
            "shell" => self.execute_shell(&processed_content),
            "api" => self.execute_api(&processed_content),
            _ => Ok(processed_content),
        };
        
        // Remove block from processing list
        self.processing_blocks.retain(|b| b != name);
        
        // Handle execution result
        match result {
            Ok(output) => {
                // Store output
                self.outputs.insert(name.to_string(), output.clone());
                
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
    
    // Process variable references like ${block_name}
    pub fn process_variable_references(&self, content: &str) -> String {
        self.process_variable_references_internal(content, &mut Vec::new())
    }
    
    // Internal implementation that tracks processing variables to detect circular references
    fn process_variable_references_internal(&self, content: &str, processing_vars: &mut Vec<String>) -> String {
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
            
            // Try to get the value from outputs
            let value = if let Some(output) = self.outputs.get(&var_name) {
                output.clone()
            } else if let Some(fallback_name) = self.fallbacks.get(&var_name) {
                // Try fallback if available
                if let Some(fallback_output) = self.outputs.get(fallback_name) {
                    fallback_output.clone()
                } else {
                    // No value found, leave the reference as is
                    continue;
                }
            } else {
                // No value or fallback found, leave the reference as is
                continue;
            };
            
            // Replace the reference with its value
            let var_ref = format!("${{{}}}", var_name);
            
            // Check if the value itself contains variable references
            if value.contains("${") {
                // Add this variable to the processing list to detect circular references
                processing_vars.push(var_name.clone());
                
                // Recursively process nested references
                let processed_value = self.process_variable_references_internal(&value, processing_vars);
                result = result.replace(&var_ref, &processed_value);
                
                // Remove from processing list
                processing_vars.retain(|v| v != &var_name);
            } else {
                // Simple replacement
                result = result.replace(&var_ref, &value);
            }
        }
        
        result
    }
    
    // Execute different types of blocks
    
    pub fn execute_python(&self, code: &str) -> Result<String, ExecutorError> {
        let child = Command::new("python")
            .arg("-c")
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
    
    // Generate a results block for an executed block
    pub fn generate_results_block(&self, block: &Block, output: &str, format_type: Option<&str>) -> Block {
        let mut results_block = Block::new("results", None, output);
        
        // Add "for" modifier pointing to the original block
        if let Some(block_name) = &block.name {
            results_block.add_modifier("for", block_name);
        }
        
        // Set format if specified or determine automatically
        let format = format_type.unwrap_or_else(|| self.determine_format_from_content(output));
        results_block.add_modifier("format", format);
        
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
    pub fn determine_format(&self, block: &Block) -> String{
        if let Some(format) = block.get_modifier("format") {
            format.to_string()
        } else {
            self.determine_format_from_content(&block.content).to_string()
        }
    }
    
    // Auto-detect content format based on its structure
    pub fn determine_format_from_content(&self, content: &str) -> &str {
        let trimmed = content.trim();
        
        // Check if it's JSON
        if (trimmed.starts_with('{') && trimmed.ends_with('}')) || 
           (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            return "json";
        }
        
        // Check if it's CSV
        if trimmed.contains(',') && 
           trimmed.lines().count() > 1 && 
           trimmed.lines().all(|line| line.contains(',')) {
            return "csv";
        }
        
        // Check if it's Markdown (contains common MD markers)
        if trimmed.contains('#') || 
           trimmed.contains("```") || 
           (trimmed.contains('|') && trimmed.contains('-')) {
            return "markdown";
        }
        
        // Default to plain text
        "plain"
    }
    
    // Apply trim modifier to results content
    pub fn apply_trim(&self, block: &Block, content: &str) -> String {
        if let Some(trim_value) = block.get_modifier("trim") {
            if trim_value == "true" {
                return content.trim().to_string();
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
                        return lines[..max_lines].join("\n");
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
        
        truncated
    }
    
    // Update the execution method to automatically create results blocks
    pub fn execute_block_with_results(&mut self, name: &str) -> Result<Block, ExecutorError> {
        match self.execute_block(name) {
            Ok(output) => {
                // Get the original block
                let block = self.blocks.get(name).unwrap().clone();
                
                // Create a results block
                let results_block = self.generate_results_block(&block, &output, None);
                
                // Process the results content
                let processed_content = self.process_results_content(&results_block, &output);
                
                // Create the final results block with processed content
                Ok(self.generate_results_block(&block, &processed_content, None))
            },
            Err(err) => {
                // Get the original block
                if let Some(block) = self.blocks.get(name) {
                    // Create an error results block
                    let _error_block = self.generate_error_results_block(block, &err.to_string());
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
        
        let mut updated_doc = self.current_document.clone();
        
        // Replace response blocks with execution results
        for (name, output) in &self.outputs {
            // Very simple replacement - in a real implementation, this would be more robust
            let response_marker = format!("[response for:{}]", name);
            let response_replacement = format!("[response for:{}]\n{}\n[/response]", name, output);
            
            updated_doc = updated_doc.replace(&response_marker, &response_replacement);
        }
        
        Ok(updated_doc)
    }
}
// Implement Default for MetaLanguageExecutor
impl Default for MetaLanguageExecutor {
    fn default() -> Self {
        Self::new()
    }
}
