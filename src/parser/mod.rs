use pest::Parser;
use pest_derive::Parser;
use anyhow::Result;
use thiserror::Error;

// Define the parser using pest
#[derive(Parser)]
#[grammar = "parser/meta_language.pest"]
pub struct MetaLanguageParser;

// Import sub-modules
mod blocks;
mod block_parser;
mod utils;
mod block_parsers;
mod modifiers;

// Re-export important types
pub use blocks::Block;
pub use block_parser::parse_single_block;
pub use utils::extractors::{extract_name, extract_modifiers, extract_variable_references};
pub use utils::validators::check_duplicate_names;

// Define error type
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse document: {0}")]
    ParseError(String),
    
    #[error("Invalid block structure: {0}")]
    InvalidBlockStructure(String),
    
    #[error("Duplicate block name: {0}")]
    DuplicateBlockName(String),
}

// Parse a document string into blocks
pub fn parse_document(input: &str) -> Result<Vec<Block>, ParserError> {
    let mut blocks = Vec::new();
    
    // Try parsing with the full document parser first
    let result = MetaLanguageParser::parse(Rule::document, input);
    
    if let Ok(pairs) = result {
        for pair in pairs {
            match pair.as_rule() {
                Rule::document => {
                    // Process all blocks in the document
                    for block_pair in pair.into_inner() {
                        if let Rule::block = block_pair.as_rule() {
                            if let Some(block) = blocks::process_block(block_pair) {
                                blocks.push(block);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        // If full document parsing fails, try parsing blocks individually
        // This helps with nested blocks and partial documents
        let mut content = input.trim();
        
        while !content.is_empty() {
            // Find the start of the next block
            if let Some(open_bracket) = content.find('[') {
                let block_start = &content[open_bracket..];
                
                // Find the matching closing tag
                if let Some(block) = try_parse_single_block(block_start) {
                    // Successfully parsed a block
                    blocks.push(block.0);
                    
                    // Move past this block in the content
                    let consumed_len = block.1;
                    if consumed_len >= content.len() {
                        break;
                    }
                    content = content[consumed_len..].trim_start();
                } else {
                    // Skip to the next potential block start
                    content = &content[open_bracket + 1..];
                }
            } else {
                // No more blocks found
                break;
            }
        }
        
        if blocks.is_empty() {
            return Err(ParserError::ParseError(format!("Failed to parse block: {}", input)));
        }
    }
    
    // Check for duplicate block names
    check_duplicate_names(&blocks)?;
    
    Ok(blocks)
}

// Try to parse a single block from the start of the content
// Returns the block and how many characters were consumed
fn try_parse_single_block(content: &str) -> Option<(Block, usize)> {
    // Check if this is a section block which can contain nested blocks
    if content.starts_with("[section:") {
        return try_parse_section_block(content);
    }
    
    // Otherwise, try to parse a regular block
    if let Ok(block) = block_parser::parse_single_block(content) {
        let end_pos = find_block_end(content, &block.block_type)?;
        return Some((block, end_pos));
    }
    
    None
}

// Find the end position of a block
fn find_block_end(content: &str, block_type: &str) -> Option<usize> {
    let close_tag = format!("[/{}", block_type);
    if let Some(close_pos) = content.find(&close_tag) {
        if let Some(end_pos) = content[close_pos..].find(']') {
            return Some(close_pos + end_pos + 1);
        }
    }
    None
}

// Parse a section block which can contain nested blocks
fn try_parse_section_block(content: &str) -> Option<(Block, usize)> {
    // Extract section type
    let start = content.find("[section:")?;
    let type_start = start + 9;
    let type_end = content[type_start..].find(' ').map(|pos| type_start + pos)
        .or_else(|| content[type_start..].find(']').map(|pos| type_start + pos))?;
    
    let section_type = content[type_start..type_end].trim();
    let block_type = format!("section:{}", section_type);
    
    // Find where this section ends
    let close_tag = format!("[/section:{}", section_type);
    let close_pos = content.find(&close_tag)?;
    let end_pos = content[close_pos..].find(']').map(|pos| close_pos + pos + 1)?;
    
    // Extract name and content
    let open_end = content.find(']')? + 1;
    let section_content = content[open_end..close_pos].trim();
    
    // Extract name
    let mut name = None;
    if let Some(name_pos) = content[start..open_end].find("name:") {
        let name_start = start + name_pos + 5;
        let name_end = content[name_start..open_end].find(' ').map(|pos| name_start + pos)
            .or_else(|| content[name_start..open_end].find(']').map(|pos| name_start + pos))?;
        name = Some(content[name_start..name_end].trim().to_string());
    }
    
    // Create the block
    let mut block = Block::new(&block_type, name.as_deref(), section_content);
    
    // Parse child blocks from the content
    if let Ok(child_blocks) = parse_document(section_content) {
        for child in child_blocks {
            block.add_child(child);
        }
    }
    
    Some((block, end_pos))
}
