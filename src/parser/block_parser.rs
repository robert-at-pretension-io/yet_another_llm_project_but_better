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
    // Extract the block type from the opening tag
    if let Some((base_type, full_type)) = extract_block_type(content) {
        // Check for matching closing tag - try both the full type and base type
        let full_close_tag = format!("[/{}", full_type);
        let base_close_tag = format!("[/{}", base_type);
        
        // First check if the full type closing tag exists
        if content.contains(&full_close_tag) {
            return true;
        }
        
        // If not found, check if the base type closing tag exists
        // This handles cases where a block might be closed with just the base type
        return content.contains(&base_close_tag);
    }
    
    false
}

// Validate block structure before parsing
pub fn validate_block_structure(content: &str) -> Result<(), ParserError> {
    if !has_matching_closing_tag(content) {
        // Try to extract the block type for a better error message
        if let Some((base_type, _)) = extract_block_type(content) {
            return Err(ParserError::InvalidBlockStructure(
                format!("Missing closing tag for block type: {}", base_type)
            ));
        } else {
            return Err(ParserError::InvalidBlockStructure("Missing or invalid closing tag".to_string()));
        }
    }
    
    Ok(())
}

// Extract the block type from a block opening tag
// This handles all variations of block syntax:
// - Simple blocks: [blocktype]
// - Blocks with subtypes: [blocktype:subtype]
// - Blocks with modifiers: [blocktype name:value]
// - Blocks with specific modifiers: [blocktype for:value]
pub fn extract_block_type(content: &str) -> Option<(String, String)> {
    // Find the opening bracket
    if let Some(open_start) = content.find('[') {
        let after_open = &content[open_start + 1..];
        
        // Find where the block type ends (at a space, colon, or closing bracket)
        if let Some(type_end) = after_open.find(|c: char| c == ' ' || c == ':' || c == ']') {
            let block_type = &after_open[..type_end];
            
            // If we have a colon immediately after the block type, it's a subtype
            let full_type = if after_open.len() > type_end && after_open.as_bytes()[type_end] == b':' {
                // Find the end of the subtype (at a space or closing bracket)
                if let Some(subtype_end) = after_open[type_end + 1..].find(|c: char| c == ' ' || c == ']') {
                    let subtype = &after_open[type_end + 1..type_end + 1 + subtype_end];
                    format!("{}:{}", block_type, subtype)
                } else {
                    // If no space or closing bracket, use everything up to the end
                    let remaining = &after_open[type_end + 1..];
                    format!("{}:{}", block_type, remaining)
                }
            } else {
                block_type.to_string()
            };
            
            // Extract the base type (before any colon)
            let base_type = if let Some(colon_pos) = full_type.find(':') {
                full_type[..colon_pos].to_string()
            } else {
                full_type.clone()
            };
            
            return Some((base_type, full_type));
        }
    }
    
    None
}
