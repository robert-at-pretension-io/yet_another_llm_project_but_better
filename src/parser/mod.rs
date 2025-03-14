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
pub use self::blocks::Block;
pub use block_parser::{parse_single_block, extract_block_type};
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
                
                // Check for template invocation with [@template_name] syntax
                if block_start.starts_with("[@") {
                    if let Some(close_bracket) = block_start.find(']') {
                        // Extract template name and modifiers
                        let template_text = &block_start[2..close_bracket];
                        
                        // Split by space to separate name and modifiers
                        let parts: Vec<&str> = template_text.split_whitespace().collect();
                        if !parts.is_empty() {
                            let template_name = parts[0];
                            
                            // Create a template invocation block with "invoke-" prefix for the name
                            let mut block = Block::new("template_invocation", Some(&format!("invoke-{}", template_name)), "");
                            
                            // Add the original template name as a modifier
                            block.add_modifier("template", template_name);
                            
                            // Process modifiers (parameters)
                            for part in &parts[1..] {
                                if let Some(colon_pos) = part.find(':') {
                                    let key = &part[0..colon_pos];
                                    let value = &part[colon_pos+1..];
                                    
                                    // Handle quoted values
                                    let clean_value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                                        &value[1..value.len()-1]
                                    } else {
                                        value
                                    };
                                    
                                    block.add_modifier(key, clean_value);
                                }
                            }
                            
                            // Find the closing tag [/@template_name]
                            let closing_tag = format!("[/@{}]", template_name);
                            if let Some(closing_pos) = content[open_bracket + close_bracket + 1..].find(&closing_tag) {
                                // Extract content between opening and closing tags
                                let content_start = open_bracket + close_bracket + 1;
                                let content_end = content_start + closing_pos;
                                let template_content = &content[content_start..content_end].trim();
                                
                                // Set the content
                                block.content = template_content.to_string();
                                
                                // Move past this template invocation including closing tag
                                content = &content[content_end + closing_tag.len()..].trim_start();
                            } else {
                                // No closing tag found, treat as self-closing
                                block.content = "".to_string();
                                content = &content[open_bracket + close_bracket + 1..].trim_start();
                            }
                            
                            blocks.push(block);
                            continue;
                        }
                    }
                }
                
                // Find the matching closing tag for regular blocks
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
                    // Failed to parse a block - likely missing closing tag
                    // Extract the block type to provide a better error message
                    if let Some(block_type_end) = block_start.find(']') {
                        let block_type = &block_start[1..block_type_end];
                        if block_type.contains(' ') {
                            let block_type = block_type.split_whitespace().next().unwrap_or("");
                            return Err(ParserError::InvalidBlockStructure(
                                format!("Missing closing tag for block type: {}", block_type)
                            ));
                        } else {
                            return Err(ParserError::InvalidBlockStructure(
                                format!("Missing closing tag for block: {}", block_type)
                            ));
                        }
                    }
                    
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
    
    // Process nested sections - ensure children are properly attached to parents
    let mut top_level_blocks = Vec::new();
    let mut section_blocks = Vec::new();
    
    // First, identify all section blocks
    for block in blocks {
        if block.block_type.starts_with("section:") {
            section_blocks.push(block);
        } else {
            top_level_blocks.push(block);
        }
    }
    
    // Check for duplicate block names
    check_duplicate_names(&top_level_blocks)?;
    
    Ok(top_level_blocks)
}

// Try to parse a single block from the start of the content
// Returns the block and how many characters were consumed
fn try_parse_single_block(content: &str) -> Option<(Block, usize)> {
    // Trim leading whitespace for more reliable detection
    let trimmed_content = content.trim_start();
    
    // Check if this is a section block which can contain nested blocks
    // Make sure we're checking the trimmed content for the section prefix
    if trimmed_content.starts_with("[section:") {
        return try_parse_section_block(content);
    }
    
    // Extract the block type
    if let Some(block_type) = extract_base_block_type(content) {
        // Try to parse using the extracted block type
        if let Ok(block) = block_parser::parse_single_block(content) {
            if let Some(end_pos) = find_block_end(content, &block.block_type) {
                return Some((block, end_pos));
            }
        }
    }
    
    // Otherwise, try to parse a regular block
    if let Ok(block) = block_parser::parse_single_block(content) {
        if let Some(end_pos) = find_block_end(content, &block.block_type) {
            return Some((block, end_pos));
        } else {
            // If we can't find the closing tag, this is an invalid block structure
            return None;
        }
    }
    
    None
}

// Helper function to extract the base block type from content
fn extract_base_block_type(content: &str) -> Option<String> {
    // First check if it's a section block
    let trimmed_content = content.trim_start();
    if trimmed_content.starts_with("[section:") {
        // Extract the section type
        let section_start = trimmed_content.find("[section:")? + 9;
        let section_end = trimmed_content[section_start..].find(']')
            .or_else(|| trimmed_content[section_start..].find(' '))?;
        
        let section_type = trimmed_content[section_start..section_start + section_end].trim();
        return Some(format!("section:{}", section_type));
    }
    
    // Otherwise use the standard extraction
    if let Some((base_type, _)) = block_parser::extract_block_type(content) {
        return Some(base_type);
    }
    None
}

// Find the end position of a block
fn find_block_end(content: &str, block_type: &str) -> Option<usize> {
    // Extract the base type (before any colon)
    let base_type = if let Some(colon_pos) = block_type.find(':') {
        &block_type[0..colon_pos]
    } else {
        block_type
    };
    
    // Try with the full block type first
    let full_close_tag = format!("[/{}", block_type);
    if let Some(close_pos) = content.find(&full_close_tag) {
        if let Some(end_pos) = content[close_pos..].find(']') {
            return Some(close_pos + end_pos + 1);
        }
    }
    
    // If that fails and we have a subtype, try with just the base type
    if block_type != base_type {
        let base_close_tag = format!("[/{}", base_type);
        if let Some(close_pos) = content.find(&base_close_tag) {
            if let Some(end_pos) = content[close_pos..].find(']') {
                return Some(close_pos + end_pos + 1);
            }
        }
    }
    
    None
}

// Parse a section block which can contain nested blocks
fn try_parse_section_block(content: &str) -> Option<(Block, usize)> {
    // Trim leading whitespace for more reliable detection
    let trimmed_content = content.trim_start();
    
    // Extract section type
    let start = trimmed_content.find("[section:")?;
    let type_start = start + 9;
    let type_end = trimmed_content[type_start..].find(' ').map(|pos| type_start + pos)
        .or_else(|| trimmed_content[type_start..].find(']').map(|pos| type_start + pos))?;
    
    let section_type = trimmed_content[type_start..type_end].trim();
    let block_type = format!("section:{}", section_type);
    
    // Adjust for the original content's whitespace
    let whitespace_offset = content.len() - trimmed_content.len();
    
    // Find where this section ends
    let close_tag = format!("[/section:{}", section_type);
    let close_pos = trimmed_content.find(&close_tag)?;
    let end_pos = trimmed_content[close_pos..].find(']').map(|pos| close_pos + pos + 1)?;
    
    // Extract name and content
    let open_end = trimmed_content[start..].find(']')? + start + 1;
    let section_content = trimmed_content[open_end..close_pos].trim();
    
    // Extract name
    let mut name = None;
    if let Some(name_pos) = trimmed_content[start..open_end].find("name:") {
        let name_start = start + name_pos + 5;
        let name_end = trimmed_content[name_start..open_end].find(' ').map(|pos| name_start + pos)
            .or_else(|| trimmed_content[name_start..open_end].find(']').map(|pos| name_start + pos))?;
        name = Some(trimmed_content[name_start..name_end].trim().to_string());
    }
    
    // Create the block
    let mut block = Block::new(&block_type, name.as_deref(), "");
    
    // Parse child blocks from the content
    let mut remaining_content = section_content;
    
    while !remaining_content.is_empty() {
        // Skip any whitespace or text before the next block
        if let Some(block_start) = remaining_content.find('[') {
            let block_content = &remaining_content[block_start..];
            
            // Try to parse a nested block
            if let Some((child_block, consumed)) = try_parse_single_block(block_content) {
                // Add the child block
                block.add_child(child_block);
                
                // Move past this block
                if block_start + consumed >= remaining_content.len() {
                    break;
                }
                remaining_content = &remaining_content[block_start + consumed..];
            } else {
                // If we couldn't parse a block, move ahead one character and try again
                if block_start + 1 >= remaining_content.len() {
                    break;
                }
                remaining_content = &remaining_content[block_start + 1..];
            }
        } else {
            // No more block starts found
            break;
        }
    }
    
    // Store the original content as well (for reference)
    block.content = section_content.to_string();
    
    // Return with adjusted end position to account for leading whitespace
    Some((block, end_pos + whitespace_offset))
}
