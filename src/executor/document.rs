use std::collections::HashMap;
use crate::executor::error::ExecutorError;
use crate::executor::state::ExecutorState;
use crate::executor::cache::CacheManager;

/// Handles document updates with execution results
pub struct DocumentUpdater<'a> {
    state: &'a ExecutorState,
    debug_enabled: bool,
}

pub struct BlockReplacement {
    original: String,
    replacement: String,
}

impl<'a> DocumentUpdater<'a> {
    pub fn new(state: &'a ExecutorState) -> Self {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        Self { state, debug_enabled }
    }
    
    /// Update a document with execution results
    pub fn update_document(&self) -> Result<String, ExecutorError> {
        // Start with the original document content
        let mut updated_content = self.state.current_document.clone();
        
        // Keep track of replacements to avoid duplicates or conflicts
        let mut replacements = HashMap::new();
        let mut update_count = 0;
        
        if self.debug_enabled {
            println!("DEBUG: Starting document update process");
        }

        // Process each block that has output
        for (name, output) in &self.state.outputs {
            // Skip special outputs like errors
            if name.ends_with("_error") {
                continue;
            }
            
            // For results blocks, get the base block name
            let is_results_block = name.ends_with(".results") || name.ends_with("_results");
            let base_block_name = if is_results_block {
                if name.ends_with(".results") {
                    name.trim_end_matches(".results")
                } else {
                    name.trim_end_matches("_results")
                }
            } else {
                name
            };
            
            if self.debug_enabled {
                println!("DEBUG: Processing output '{}' (base name: '{}')", name, base_block_name);
            }

            // Only replace blocks that exist in the document
            if let Some(block) = self.state.blocks.get(base_block_name) {
                // Skip results blocks for direct processing
                if is_results_block {
                    continue;
                }
                
                // Skip blocks that aren't cacheable unless explicitly set to cache
                let should_update = CacheManager::is_cacheable(block) && block.content != *output;
                
                if should_update {
                    update_count += 1;
                    
                    // Handle different block types (code/shell need CDATA)
                    let is_cdata_block = block.block_type.starts_with("code") || 
                                         block.block_type == "shell" || 
                                         block.block_type == "api";
                    
                    // Find the block in the document
                    if let Some(start_tag) = updated_content.find(&format!("<meta:{}", &block.block_type)) {
                        if let Some(end_tag) = updated_content[start_tag..]
                            .find(&format!("</meta:{}>", &block.block_type))
                        {
                            let end_pos = start_tag + end_tag + format!("</meta:{}>", &block.block_type).len();
                            let full_tag = &updated_content[start_tag..end_pos];

                            // Only replace if we haven't already replaced this
                            if !replacements.contains_key(full_tag) {
                                // Create replacement with updated content
                                if let Some(replacement) = self.create_replacement(full_tag, output, is_cdata_block) {
                                    replacements.insert(full_tag.to_string(), replacement);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply all replacements
        for (original, replacement) in &replacements {
            updated_content = updated_content.replace(original, replacement);
        }

        if self.debug_enabled {
            println!("DEBUG: Applied {} updates to document", update_count);
        } else if update_count > 0 {
            println!("Applied {} updates to document", update_count);
        }

        Ok(updated_content)
    }
    
    /// Create a replacement block with updated content
    fn create_replacement(&self, original: &str, new_content: &str, is_cdata_block: bool) -> Option<String> {
        let mut replacement = original.to_string();

        // Find the content part (after ">")
        if let Some(content_start) = replacement.find('>') {
            let content_start = content_start + 1;
            if let Some(content_end) = replacement[content_start..].find("</meta:") {
                // Check if we need to preserve CDATA
                let formatted_output = if is_cdata_block {
                    // If the original had CDATA, preserve it
                    if replacement[content_start..content_start+content_end].contains("<![CDATA[") {
                        format!("<![CDATA[\n{}\n  ]]>", new_content)
                    } else {
                        // No CDATA found, but we should add it for safety
                        format!("<![CDATA[\n{}\n  ]]>", new_content)
                    }
                } else {
                    new_content.to_string()
                };
                
                // Replace just the content part
                let prefix = &replacement[0..content_start];
                let suffix = &replacement[content_start + content_end..];
                replacement = format!("{}{}{}", prefix, formatted_output, suffix);
                
                return Some(replacement);
            }
        }
        
        None
    }
}
