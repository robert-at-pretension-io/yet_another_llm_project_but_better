use std::collections::HashMap;
use crate::parser::{ParserError, Block};

// Check for duplicate block names
pub fn check_duplicate_names(blocks: &[Block]) -> Result<(), ParserError> {
    let mut block_names = HashMap::new();
    for block in blocks {
        if let Some(name) = &block.name {
            if block_names.contains_key(name) {
                return Err(ParserError::DuplicateBlockName(name.clone()));
            }
            block_names.insert(name.clone(), true);
        }
    }
    Ok(())
}
