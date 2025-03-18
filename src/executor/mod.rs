mod error;
mod state;
mod cache;
mod resolver;
mod document;
pub mod runners;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::parser::{parse_document, Block};

// Re-export error types
pub use error::ExecutorError;
pub use state::ExecutorState;
pub use resolver::ReferenceResolver;
pub use document::DocumentUpdater;
pub use cache::CacheManager;
pub use runners::{BlockRunner, RunnerRegistry};

/// Main executor for processing Meta Programming Language documents
pub struct MetaLanguageExecutor {
    // State and runners
    pub state: ExecutorState,
    runners: RunnerRegistry,
    
    // Backward compatibility fields - direct access to state for tests
    pub blocks: HashMap<String, Block>,
    pub outputs: HashMap<String, String>,
    pub fallbacks: HashMap<String, String>,
    pub cache: HashMap<String, (String, Instant)>,
    pub current_document: String,
    pub processing_blocks: Vec<String>,
    pub instance_id: String,
}

impl MetaLanguageExecutor {
    /// Create a new executor instance
    pub fn new() -> Self {
        let state = ExecutorState::new();
        
        // Initialize compatibility fields with empty values
        Self {
            blocks: HashMap::new(),
            outputs: HashMap::new(),
            fallbacks: HashMap::new(),
            cache: HashMap::new(),
            current_document: String::new(),
            processing_blocks: Vec::new(),
            instance_id: state.instance_id.clone(),
            state,
            runners: RunnerRegistry::new(),
        }
    }
    
    /// Process a document and extract blocks
    pub fn process_document(&mut self, content: &str) -> Result<(), ExecutorError> {
        println!("Processing document with executor: {}", self.state.instance_id);
        
        // Set environment variable to preserve variable references in original block content
        std::env::set_var("LLM_PRESERVE_REFS", "1");
        
        // Parse the document
        let blocks = parse_document(content)
            .map_err(|e| ExecutorError::ExecutionFailed(e.to_string()))?;
        
        println!("Parsed {} blocks from document", blocks.len());
        
        // Store the current outputs before clearing
        let previous_outputs = self.state.outputs.clone();
        
        // Reset state for new document while keeping cache
        self.state.reset(content);
        
        // Update compatibility fields
        self.current_document = content.to_string();
        self.blocks.clear();
        self.outputs.clear();
        self.fallbacks.clear();
        self.processing_blocks.clear();
        
        // Register all blocks and identify fallbacks
        self.register_blocks(&blocks);
        
        // Restore previous responses
        self.state.restore_responses(previous_outputs);
        
        // Sync with compatibility fields
        self.outputs = self.state.outputs.clone();
        
        // Process all references in blocks
        self.process_references()?;
        
        // Process executable blocks that don't depend on other blocks
        for block in blocks {
            if self.is_executable_block(&block) && !self.has_explicit_dependency(&block) {
                if let Some(name) = &block.name {
                    println!("Executing independent block: '{}'", name);
                    self.execute_block(name)?;
                }
            }
        }
        
        // Final sync of compatibility fields
        self.blocks = self.state.blocks.clone();
        self.outputs = self.state.outputs.clone();
        self.fallbacks = self.state.fallbacks.clone();
        self.current_document = self.state.current_document.clone();
        self.processing_blocks = self.state.processing_blocks.clone();
        
        Ok(())
    }
    
    /// Register blocks from parsed document
    fn register_blocks(&mut self, blocks: &[Block]) {
        for (index, block) in blocks.iter().enumerate() {
            // Generate a name for the block if it doesn't have one
            let block_key = if let Some(name) = &block.name {
                name.clone()
            } else {
                // Generate a unique name based on block type and index
                let generated_name = format!("{}_{}", block.block_type, index);
                generated_name
            };
            
            println!("Registering block '{}' of type '{}'", block_key, block.block_type);
            self.state.blocks.insert(block_key.clone(), block.clone());
            
            // Update compatibility fields
            self.blocks.insert(block_key.clone(), block.clone());
            
            // Check if this is a fallback block
            if let Some(name) = &block.name {
                if name.ends_with("-fallback") {
                    let original_name = name.trim_end_matches("-fallback");
                    self.state.fallbacks.insert(original_name.to_string(), name.clone());
                    
                    // Update compatibility fields
                    self.fallbacks.insert(original_name.to_string(), name.clone());
                }
                
                // Store content of data blocks directly in outputs
                if block.block_type == "data" {
                    self.state.outputs.insert(name.clone(), block.content.clone());
                    
                    // Update compatibility fields
                    self.outputs.insert(name.clone(), block.content.clone());
                }
            }
        }
    }
    
    /// Process all variable references in blocks
    fn process_references(&mut self) -> Result<(), ExecutorError> {
        let resolver = ReferenceResolver::new(&self.state);
        
        // Collect all block names upfront
        let all_block_names: Vec<String> = self.state.blocks.keys().cloned().collect();
        
        // Process data blocks first (may contain references to other data)
        let data_block_names: Vec<String> = self.state.blocks.iter()
            .filter(|(_, block)| block.block_type == "data" || block.block_type.starts_with("data:"))
            .map(|(name, _)| name.clone())
            .collect();
            
        let mut blocks = self.state.blocks.clone();
        let mut outputs = self.state.outputs.clone();
        
        // Process data blocks first
        resolver.process_blocks(&mut blocks, &mut outputs, &data_block_names, "data")?;
        
        // Process non-data blocks
        let non_data_blocks: Vec<String> = all_block_names.iter()
            .filter(|name| !data_block_names.contains(name))
            .cloned()
            .collect();
            
        resolver.process_blocks(&mut blocks, &mut outputs, &non_data_blocks, "non-data")?;
        
        // Final pass for any remaining references
        resolver.process_blocks(&mut blocks, &mut outputs, &all_block_names, "final")?;
        
        // Update state
        self.state.blocks = blocks;
        self.state.outputs = outputs;
        
        // Update backward compatibility fields
        self.blocks = self.state.blocks.clone();
        self.outputs = self.state.outputs.clone();
        self.fallbacks = self.state.fallbacks.clone();
        
        Ok(())
    }
    
    /// Process variable references in content (for backward compatibility)
    pub fn process_variable_references(&self, content: &str) -> Result<String, ExecutorError> {
        let resolver = ReferenceResolver::new(&self.state);
        resolver.process_content(content)
    }
    
    /// Process variable references in an XML element tree (for backward compatibility)
    pub fn process_element_references(&self, element: &mut xmltree::Element, flatten_references: bool) -> Result<(), ExecutorError> {
        let resolver = ReferenceResolver::new(&self.state);
        resolver.process_element_references(element)?;
        
        // Flatten reference elements if requested
        if flatten_references {
            self.flatten_references(element);
        }
        
        Ok(())
    }
    
    /// Helper method to flatten reference elements into their text content
    fn flatten_references(&self, element: &mut xmltree::Element) {
        let mut new_children = Vec::new();
        let children = element.children.clone();
        
        for child in children {
            match child {
                xmltree::XMLNode::Element(mut child_elem) => {
                    // Recursively process children
                    self.flatten_references(&mut child_elem);
                    
                    // If this was a reference element, extract its text content directly
                    if child_elem.name == "meta:reference" || child_elem.name.ends_with(":reference") {
                        if child_elem.children.len() == 1 {
                            if let Some(xmltree::XMLNode::Text(text)) = child_elem.children.first() {
                                new_children.push(xmltree::XMLNode::Text(text.clone()));
                                continue;
                            }
                        }
                    }
                    
                    new_children.push(xmltree::XMLNode::Element(child_elem));
                },
                xmltree::XMLNode::Text(text) => {
                    new_children.push(xmltree::XMLNode::Text(text));
                },
                other => {
                    new_children.push(other);
                }
            }
        }
        
        // Replace the element's children with the processed ones
        element.children = new_children;
    }
    
    /// Update the document with execution results
    pub fn update_document(&self) -> Result<String, ExecutorError> {
        let updater = DocumentUpdater::new(&self.state);
        updater.update_document()
    }
    
    /// Helper method to register a runner (mainly for testing)
    pub fn register_runner(&mut self, runner: Box<dyn BlockRunner>) {
        self.runners.register(runner);
    }
    
    /// Execute a block by name
    pub fn execute_block(&mut self, name: &str) -> Result<String, ExecutorError> {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Executing block: '{}'", name);
        }
        
        // Check for circular dependencies
        if self.state.processing_blocks.contains(&name.to_string()) {
            return Err(ExecutorError::CircularDependency(name.to_string()));
        }
        
        // Check if block exists
        let block = match self.state.blocks.get(name) {
            Some(b) => b.clone(),
            None => {
                return Err(ExecutorError::BlockNotFound(name.to_string()));
            }
        };
        
        // Check if result is cached
        if CacheManager::is_cacheable(&block) {
            if let Some((result, timestamp)) = self.state.cache.get(name) {
                // Check if cache is still valid (within timeout)
                let now = Instant::now();
                let timeout = CacheManager::get_timeout(&block);
                let elapsed = now.duration_since(*timestamp);
                
                if elapsed < timeout {
                    if debug_enabled {
                        println!(
                            "DEBUG: Using cached result for '{}' (age: {:.2}s, timeout: {}s)",
                            name,
                            elapsed.as_secs_f64(),
                            timeout.as_secs()
                        );
                    }
                    return Ok(result.clone());
                }
            }
        }
        
        // Mark block as being processed
        self.state.processing_blocks.push(name.to_string());
        
        // Update compatibility fields
        self.processing_blocks = self.state.processing_blocks.clone();
        
        // Execute dependencies first
        self.execute_dependencies(&block, name)?;
        
        // Get the most up-to-date block content
        let block_content = if let Some(updated_block) = self.state.blocks.get(name) {
            updated_block.content.clone()
        } else {
            block.content.clone()
        };
        
        // Process variable references 
        let resolver = ReferenceResolver::new(&self.state);
        let processed_content = resolver.process_content(&block_content)?;
        
        // Update the block with processed content
        if let Some(updated_block) = self.state.blocks.get_mut(name) {
            updated_block.content = processed_content.clone();
        }
        
        // Update compatibility field
        if let Some(compat_block) = self.blocks.get_mut(name) {
            compat_block.content = processed_content.clone();
        }
        
        // Find appropriate runner and execute
        let result = if let Some(runner) = self.runners.find_runner(&block) {
            // We have a specific runner for this block type
            runner.execute(name, &block, &mut self.state)
        } else {
            // Default handling for blocks without specific runners
            Ok(processed_content)
        };
        
        // Remove from processing list
        self.state.processing_blocks.retain(|b| b != name);
        
        // Update compatibility fields
        self.processing_blocks = self.state.processing_blocks.clone();
        
        // Handle execution result
        match result {
            Ok(output) => {
                // Store output
                self.state.store_block_output(name, output.clone());
                
                // Update compatibility fields
                self.outputs.insert(name.to_string(), output.clone());
                let results_key = format!("{}.results", name);
                self.outputs.insert(results_key.clone(), output.clone());
                let alt_results_key = format!("{}_results", name);
                self.outputs.insert(alt_results_key.clone(), output.clone());
                
                // Cache if needed
                if CacheManager::is_cacheable(&block) {
                    self.state.cache.insert(name.to_string(), (output.clone(), Instant::now()));
                    self.cache.insert(name.to_string(), (output.clone(), Instant::now()));
                }
                
                Ok(output)
            },
            Err(e) => {
                // Store error
                self.state.store_error(name, &e.to_string());
                
                // Update compatibility fields
                let error_key = format!("{}_error", name);
                self.outputs.insert(error_key, e.to_string());
                
                // Try fallback
                if let Some(fallback_name) = self.state.fallbacks.get(name) {
                    println!("Block '{}' failed, using fallback: {}", name, fallback_name);
                    self.execute_block(&fallback_name.clone())
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Execute dependencies for a block
    fn execute_dependencies(&mut self, block: &Block, block_name: &str) -> Result<(), ExecutorError> {
        for (key, value) in &block.modifiers {
            if key == "depends" || key == "requires" || key == "if" {
                let dependency_type = if key == "if" { "condition" } else { "dependency" };
                
                println!("Block '{}' has {} '{}', executing it first", block_name, dependency_type, value);
                self.execute_block(value)?;
            }
        }
        Ok(())
    }
    
    /// Check if a block is executable
    pub fn is_executable_block(&self, block: &Block) -> bool {
        matches!(
            block.block_type.as_str(),
            "code:python" | "code:javascript" | "code:rust" | "shell" | "api" | "question" | "conditional"
        )
    }
    
    /// Check if a block has explicit dependencies
    pub fn has_explicit_dependency(&self, block: &Block) -> bool {
        block.modifiers.iter().any(|(key, _)| key == "depends" || key == "requires" || key == "if")
    }
    
    /// Check if a block has a fallback (for backward compatibility)
    pub fn has_fallback(&self, name: &str) -> bool {
        self.fallbacks.contains_key(name)
    }
    
    /// Get timeout for a block (for backward compatibility)
    pub fn get_timeout(&self, block: &Block) -> Duration {
        CacheManager::get_timeout(block)
    }
    
    /// Check if a block is cacheable (for backward compatibility)
    pub fn is_cacheable(&self, block: &Block) -> bool {
        CacheManager::is_cacheable(block)
    }
}