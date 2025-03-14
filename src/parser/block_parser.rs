// A separate module for parsing individual blocks
use crate::parser::{ParserError, Block};
use crate::parser::blocks::process_block;
use crate::parser::{Rule, MetaLanguageParser};
use pest::Parser;

// Parse a string that contains a single block
pub fn parse_single_block(input: &str) -> Result<Block, ParserError> {
    // First check if the block has a matching closing tag
    if !has_matching_closing_tag(input) {
        return Err(ParserError::InvalidBlockStructure("Missing or invalid closing tag".to_string()));
    }
    
    // Attempt to parse with pest
    let result = MetaLanguageParser::parse(Rule::block, input);
    match result {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                if let Some(block) = process_block(pair) {
                    Ok(block)
                } else {
                    Err(ParserError::ParseError("Failed to process block".to_string()))
                }
            } else {
                Err(ParserError::ParseError("Empty block".to_string()))
            }
        }
        Err(e) => Err(ParserError::ParseError(e.to_string())),
    }
}

// Helper function to validate block structure (check for matching closing tags)
fn has_matching_closing_tag(content: &str) -> bool {
    // Find the block type from the opening tag
    if let Some(open_start) = content.find('[') {
        let after_open = &content[open_start + 1..];
        
        // Handle special block types with modifiers directly after opening bracket
        // Examples: [results for:simple-calc format:plain], [error_results for:test]
        let known_block_types = ["results", "error_results", "api", "preview"];
        for block_type in known_block_types.iter() {
            if after_open.starts_with(block_type) && 
               (after_open.len() > block_type.len()) && 
               (after_open.as_bytes()[block_type.len()] == b' ') {
                // Found a known block type with space after it (indicating modifiers)
                let close_tag = format!("[/{}", block_type);
                return content.contains(&close_tag);
            }
        }
        
        // Standard block type extraction (including subtypes with colons)
        if let Some(type_end) = after_open.find(|c: char| c == ' ' || c == ']') {
            let full_block_type = &after_open[..type_end];
            
            // Handle block types with subtypes (e.g., "code:python")
            let base_type = if let Some(colon_pos) = full_block_type.find(':') {
                &full_block_type[..colon_pos]
            } else {
                full_block_type
            };
            
            // Check for matching closing tag - try both the full type and base type
            let full_close_tag = format!("[/{}", full_block_type);
            let base_close_tag = format!("[/{}", base_type);
            
            // First check if the full type closing tag exists
            if content.contains(&full_close_tag) {
                return true;
            }
            
            // If not found, check if the base type closing tag exists
            // This handles cases where a block might be closed with just the base type
            return content.contains(&base_close_tag);
        }
    }
    
    false
}

// Validate block structure before parsing
pub fn validate_block_structure(content: &str) -> Result<(), ParserError> {
    if !has_matching_closing_tag(content) {
        return Err(ParserError::InvalidBlockStructure("Missing or invalid closing tag".to_string()));
    }
    
    Ok(())
}
