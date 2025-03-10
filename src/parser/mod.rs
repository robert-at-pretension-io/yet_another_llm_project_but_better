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
                Rule::EOI => {}, // End of input, ignore
                _ => {},
            }
        }
        
        if !blocks.is_empty() {
            // Check for duplicate block names
            check_duplicate_names(&blocks)?;
            return Ok(blocks);
        }
    }
    
    // Fallback: try parsing blocks individually
    blocks.clear();
    
    // Handle multiple blocks separated by blank lines
    if input.contains("\n\n") {
        let parts = input.split("\n\n").collect::<Vec<&str>>();
        
        for part in parts {
            if !part.trim().is_empty() {
                // Try to parse as individual block
                if let Ok(block) = parse_single_block(part) {
                    blocks.push(block);
                } else {
                    // If we fail to parse, we might have syntax errors
                    return Err(ParserError::ParseError(format!("Failed to parse block: {}", part)));
                }
            }
        }
        
        if !blocks.is_empty() {
            // Check for duplicate block names
            check_duplicate_names(&blocks)?;
            return Ok(blocks);
        }
    } else {
        // Try as a single block
        if let Ok(block) = parse_single_block(input) {
            blocks.push(block);
            return Ok(blocks);
        }
    }
    
    // If above approaches fail, try the full document parser again with error reporting
    let pairs = MetaLanguageParser::parse(Rule::document, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    blocks.clear();
    
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
            Rule::EOI => {}, // End of input, ignore
            _ => {},
        }
    }
    
    // Check for duplicate block names
    check_duplicate_names(&blocks)?;
    
    Ok(blocks)
}
