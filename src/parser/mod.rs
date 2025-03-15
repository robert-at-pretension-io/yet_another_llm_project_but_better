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
pub mod document_processor;
mod xml_parser;
mod debug_utils;

// Re-export important types
pub use self::blocks::Block;
pub use block_parser::{parse_single_block, extract_block_type};
pub use utils::extractors::{extract_name, extract_modifiers};
pub use utils::validators::check_duplicate_names;
pub use xml_parser::{parse_xml_document};

// Define error type
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse document: {0}")]
    ParseError(String),
    
    #[error("Invalid block structure: {0}")]
    InvalidBlockStructure(String),
    
    #[error("Duplicate block name: {0}")]
    DuplicateBlockName(String),
    
    #[error("Invalid block type: {0}")]
    InvalidBlockType(String),
}

// List of valid block types
// Extract variable references from text
pub fn extract_variable_references(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < text.len() {
        // Look for the start of a variable reference
        if let Some(start) = text[i..].find("${") {
            let start_pos = i + start + 2; // Skip the ${ prefix
            
            // Look for the closing brace
            if let Some(end) = text[start_pos..].find('}') {
                let end_pos = start_pos + end;
                let var_name = &text[start_pos..end_pos];
                
                // Add the variable name to the result
                result.push(var_name.to_string());
                
                // Continue searching after this variable
                i = end_pos + 1;
            } else {
                // No closing brace found, stop searching
                break;
            }
        } else {
            // No more variable references found
            break;
        }
    }
    
    result
}

pub fn is_valid_block_type(block_type: &str) -> bool {
    // Check base types
    let base_types = [
        "code", "data", "shell", "visualization", "template", "variable", 
        "secret", "filename", "memory", "api", "question", "response", 
        "results", "error_results", "error", "preview", "conditional", 
        "section", "template_invocation", "error-response"
    ];
    
    // For block types with subtypes (like code:python or section:intro)
    if let Some(colon_pos) = block_type.find(':') {
        let base_type = &block_type[0..colon_pos];
        return base_types.contains(&base_type);
    }
    
    // For simple block types
    base_types.contains(&block_type)
}

// Parse a document string into blocks
pub fn parse_document(input: &str) -> Result<Vec<Block>, ParserError> {
    println!("DEBUG: parse_document called with input length: {} characters", input.len());
    println!("DEBUG: Input preview: {}", &input[..std::cmp::min(100, input.len())]);
    
    // Use only the XML parser for document parsing
    println!("DEBUG: Calling xml_parser::parse_xml_document");
    match xml_parser::parse_xml_document(input) {
        Ok(blocks) => {
            println!("DEBUG: XML parsing successful, got {} blocks", blocks.len());
            
            // Validate block types
            for (i, block) in blocks.iter().enumerate() {
                println!("DEBUG: Validating block {}: type={}, name={:?}", 
                         i, block.block_type, block.name);
                
                if !is_valid_block_type(&block.block_type) {
                    println!("DEBUG: Invalid block type: {}", block.block_type);
                    return Err(ParserError::InvalidBlockType(block.block_type.clone()));
                }
            }
            
            println!("DEBUG: All blocks validated successfully");
            Ok(blocks)
        },
        Err(err) => {
            println!("DEBUG: XML parsing failed: {}", err);
            Err(ParserError::ParseError(format!("XML parsing failed: {}", err)))
        }
    }
}
