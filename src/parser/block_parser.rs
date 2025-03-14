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
        
        // Extract block type
        if let Some(type_end) = after_open.find(|c: char| c == ' ' || c == ']') {
            let block_type = &after_open[..type_end];
            
            // Check for matching closing tag
            let close_tag = format!("[/{}", block_type);
            return content.contains(&close_tag);
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
