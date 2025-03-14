// A separate module for parsing individual blocks
use crate::parser::{ParserError, Block};
use crate::parser::block_parsers::*;
use crate::parser::modifiers::extract_modifiers;

pub fn parse_single_block(content: &str) -> Result<Block, ParserError> {
    // Check if the block has a proper closing tag
    if !has_matching_closing_tag(content) {
        return Err(ParserError::InvalidBlockStructure(
            "Missing or invalid closing tag".to_string()
        ));
    }
    
    // Continue with normal parsing logic
    // This is a placeholder for the actual parsing logic
    Err(ParserError::NotImplemented("Block parsing not fully implemented".to_string()))
}

// Helper function to check if a block has a matching closing tag
fn has_matching_closing_tag(content: &str) -> bool {
    // Extract the block type from opening tag
    if let Some(open_bracket) = content.find('[') {
        if let Some(first_space) = content[open_bracket..].find(' ') {
            let block_type = &content[open_bracket + 1..open_bracket + first_space];
            
            // Look for matching closing tag
            let close_tag = format!("[/{}", block_type);
            return content.contains(&close_tag);
        }
    }
    false
}
