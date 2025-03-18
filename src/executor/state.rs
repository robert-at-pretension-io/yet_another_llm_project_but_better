use std::collections::HashMap;
use std::time::Instant;
use crate::parser::Block;

/// Centralized state management for the executor
/// Holds blocks, outputs, and cache state
pub struct ExecutorState {
    // Document state
    pub blocks: HashMap<String, Block>,
    pub outputs: HashMap<String, String>,
    pub fallbacks: HashMap<String, String>,
    pub current_document: String,
    
    // Execution state
    pub processing_blocks: Vec<String>,
    pub instance_id: String,
    
    // Cache state
    pub cache: HashMap<String, (String, Instant)>,
}

impl ExecutorState {
    pub fn new() -> Self {
        // Generate a unique ID for this executor instance
        let instance_id = format!(
            "executor_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        Self {
            blocks: HashMap::new(),
            outputs: HashMap::new(),
            fallbacks: HashMap::new(),
            current_document: String::new(),
            processing_blocks: Vec::new(),
            instance_id,
            cache: HashMap::new(),
        }
    }
    
    /// Store output for a block and its derived result keys
    pub fn store_block_output(&mut self, name: &str, output: String) {
        // Store output with the block name
        self.outputs.insert(name.to_string(), output.clone());

        // Also store with block_name.results format
        let results_key = format!("{}.results", name);
        self.outputs.insert(results_key, output.clone());

        // Also store with block_name_results format for compatibility
        let results_key = format!("{}_results", name);
        self.outputs.insert(results_key, output.clone());

        // Update block content
        if let Some(block) = self.blocks.get_mut(name) {
            block.content = output.clone();
        }
    }
    
    /// Store error for a block
    pub fn store_error(&mut self, name: &str, error: &str) {
        // Store error with block_name_error format
        let error_key = format!("{}_error", name);
        self.outputs.insert(error_key, error.to_string());
    }
    
    /// Clear state while keeping cache intact
    pub fn reset(&mut self, new_document: &str) {
        self.blocks.clear();
        self.outputs.clear();
        self.fallbacks.clear();
        self.current_document = new_document.to_string();
        self.processing_blocks.clear();
    }
    
    /// Restore previous responses from old state
    pub fn restore_responses(&mut self, previous_outputs: HashMap<String, String>) {
        for (key, value) in previous_outputs {
            // Only restore responses, not other outputs
            if key.ends_with("_response") || key == "question_response" {
                // Check if this response isn't already in the outputs map
                if !self.outputs.contains_key(&key) {
                    self.outputs.insert(key.clone(), value);
                }
            }
        }
    }
    
    /// Check if a block has a fallback defined
    pub fn has_fallback(&self, name: &str) -> bool {
        self.fallbacks.contains_key(name)
    }
}
