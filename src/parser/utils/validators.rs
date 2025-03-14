use std::collections::HashMap;
use crate::parser::{ParserError, Block};

// Check for duplicate block names
pub fn check_duplicate_names(blocks: &[Block]) -> Result<(), ParserError> {
    let mut block_names = HashMap::new();
    for block in blocks {
        if let Some(name) = &block.name {
            // Check if this is a template invocation (which now has a different naming scheme)
            let is_template_invocation = block.block_type == "template_invocation" || 
                                        block.block_type.starts_with("template_invocation:");
            
            // Only check for duplicates if it's not a template invocation or
            // if it's a template invocation but the name is already used by another invocation
            if block_names.contains_key(name) {
                let existing_type = block_names.get(name).unwrap();
                
                // If both are template invocations, that's still a duplicate
                // If one is a template invocation and the other isn't, that's also a duplicate
                if !is_template_invocation || *existing_type == "template_invocation" {
                    return Err(ParserError::DuplicateBlockName(name.clone()));
                }
            }
            
            // Store the block type along with the name
            block_names.insert(name.clone(), if is_template_invocation { 
                "template_invocation".to_string() 
            } else { 
                block.block_type.clone() 
            });
        }
    }
    Ok(())
}
