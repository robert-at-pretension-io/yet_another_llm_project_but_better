// src/parser/block_parsers/results_blocks.rs

use crate::parser::{Block, ParserError};
use regex::Regex;

// Parse results block
pub fn parse_results_block(input: &str) -> Result<Block, ParserError> {
    // Basic validation
    if !input.trim().starts_with("[results") {
        return Err(ParserError::ParseError(format!("Not a results block: {}", input)));
    }
    
    // Extract block type and modifiers
    let opening_tag_end = input.find(']').unwrap_or(input.len());
    let opening_tag = &input[..opening_tag_end + 1];
    
    // Extract content
    let content_start = opening_tag_end + 1;
    let closing_tag = "[/results]";
    let content_end = input[content_start..].find(closing_tag)
        .map(|pos| content_start + pos)
        .unwrap_or(input.len());
    
    let content = input[content_start..content_end].trim();
    
    // Create block
    let mut block = Block::new("results", None, content);
    
    // Extract modifiers using regex for more precise matching
    let for_re = Regex::new(r"for:([^\s\]]+)").unwrap();
    if let Some(cap) = for_re.captures(opening_tag) {
        block.add_modifier("for", &cap[1]);
    }
    
    let format_re = Regex::new(r"format:([^\s\]]+)").unwrap();
    if let Some(cap) = format_re.captures(opening_tag) {
        block.add_modifier("format", &cap[1]);
    }
    
    let display_re = Regex::new(r"display:([^\s\]]+)").unwrap();
    if let Some(cap) = display_re.captures(opening_tag) {
        block.add_modifier("display", &cap[1]);
    }
    
    let max_lines_re = Regex::new(r"max_lines:([^\s\]]+)").unwrap();
    if let Some(cap) = max_lines_re.captures(opening_tag) {
        block.add_modifier("max_lines", &cap[1]);
    }
    
    let trim_re = Regex::new(r"trim:([^\s\]]+)").unwrap();
    if let Some(cap) = trim_re.captures(opening_tag) {
        block.add_modifier("trim", &cap[1]);
    }
    
    Ok(block)
}

// Parse error results block
pub fn parse_error_results_block(input: &str) -> Result<Block, ParserError> {
    // Basic validation
    if !input.trim().starts_with("[error_results") {
        return Err(ParserError::ParseError(format!("Not an error_results block: {}", input)));
    }
    
    // Extract block type and modifiers
    let opening_tag_end = input.find(']').unwrap_or(input.len());
    let opening_tag = &input[..opening_tag_end + 1];
    
    // Extract content
    let content_start = opening_tag_end + 1;
    let closing_tag = "[/error_results]";
    let content_end = input[content_start..].find(closing_tag)
        .map(|pos| content_start + pos)
        .unwrap_or(input.len());
    
    let content = input[content_start..content_end].trim();
    
    // Create block
    let mut block = Block::new("error_results", None, content);
    
    // Extract for modifier using regex for more precise matching
    let for_re = Regex::new(r"for:([^\s\]]+)").unwrap();
    if let Some(cap) = for_re.captures(opening_tag) {
        block.add_modifier("for", &cap[1]);
    }
    
    Ok(block)
}
