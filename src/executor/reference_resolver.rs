use crate::parser::Block;
use std::collections::HashMap;

/// Resolves references to other blocks in the document
pub struct ReferenceResolver {
    blocks: HashMap<String, Block>,
    outputs: HashMap<String, String>,
}

impl ReferenceResolver {
    /// Create a new ReferenceResolver with the given blocks and outputs
    pub fn new(blocks: &HashMap<String, Block>, outputs: &HashMap<String, String>) -> Self {
        Self {
            blocks: blocks.clone(),
            outputs: outputs.clone(),
        }
    }
    
    /// Resolve a reference block to its content
    pub fn resolve_reference_block(&self, block: &Block) -> Result<String, String> {
        // Get the target of this reference
        let target = match block.get_modifier("target") {
            Some(target) => target,
            None => return Err("Reference missing target attribute".to_string()),
        };
        
        println!("DEBUG: Resolving reference to target: {}", target);
        
        // Get the content to insert from either blocks or outputs
        let content = if let Some(target_block) = self.blocks.get(target) {
            println!("DEBUG: Found target block '{}' with content length: {}", 
                     target, target_block.content.len());
            target_block.content.clone()
        } else if let Some(output) = self.outputs.get(target) {
            println!("DEBUG: Found target in outputs '{}' with content length: {}", 
                     target, output.len());
            output.clone()
        } else {
            return Err(format!("Reference target '{}' not found", target));
        };
        
        // Apply modifiers (format, limit, etc.) - we'll implement this in a future step
        Ok(content)
    }
}
