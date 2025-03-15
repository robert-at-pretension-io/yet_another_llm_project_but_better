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
pub mod document_processor;
mod xml_parser;

// Re-export important types
pub use self::blocks::Block;
pub use block_parser::{parse_single_block, extract_block_type};
pub use utils::extractors::{extract_name, extract_modifiers, extract_variable_references};
pub use utils::validators::check_duplicate_names;
pub use xml_parser::{parse_xml_document, is_xml_document};

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
fn is_valid_block_type(block_type: &str) -> bool {
    // Check base types
    let base_types = [
        "code", "data", "shell", "visualization", "template", "variable", 
        "secret", "filename", "memory", "api", "question", "response", 
        "results", "error_results", "error", "preview", "conditional", 
        "section", "template_invocation"
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
    // Use only the XML parser for document parsing
    match xml_parser::parse_xml_document(input) {
        Ok(blocks) => Ok(blocks),
        Err(err) => Err(ParserError::ParseError(format!("XML parsing failed: {}", err)))
    }
}
