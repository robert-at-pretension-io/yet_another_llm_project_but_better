use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::BufRead;
use std::str;

use crate::parser::blocks::Block;
use crate::parser::ParserError;
use crate::parser::is_valid_block_type;

/// Parse an XML document into a vector of blocks
pub fn parse_xml_document(input: &str) -> Result<Vec<Block>, ParserError> {
    let mut reader = Reader::from_str(input);
    reader.trim_text(true);
    
    let mut blocks = Vec::new();
    let mut buf = Vec::new();
    
    // Track if we're inside a document element (either meta:document or document)
    let mut in_document = false;
    
    // Track current block being built
    let mut block_stack: Vec<Block> = Vec::new();
    let mut content_stack: Vec<String> = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // Convert tag name to string - compatible with quick-xml 0.28
                let name = str::from_utf8(e.name().as_ref())
                    .unwrap_or_default()
                    .to_string();
                
                // Check for document tag (with or without meta: prefix)
                if name == "meta:document" || name == "document" {
                    in_document = true;
                    continue;
                }
                
                // Only process elements inside document
                if !in_document {
                    continue;
                }
                
                // Get the block type by removing meta: prefix if present
                let block_type = if name.starts_with("meta:") {
                    name.trim_start_matches("meta:").to_string()
                } else {
                    name
                };
                
                // Check if this is a valid block type
                if is_valid_block_type(&block_type) {
                    // Extract attributes
                    let mut block_name = None;
                    let mut modifiers = Vec::new();
                    let mut final_block_type = block_type.clone();
                    
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key = str::from_utf8(attr.key.as_ref())
                                .unwrap_or_default()
                                .to_string();
                            let value = str::from_utf8(&attr.value)
                                .unwrap_or_default()
                                .to_string();
                            
                            if key == "name" {
                                block_name = Some(value);
                            } else if key == "type" && block_type == "section" {
                                // For sections, store type as a modifier
                                modifiers.push((key.clone(), value.clone()));
                            } else if key == "type" && block_type == "code" {
                                // For code blocks, handle language/type as a modifier
                                modifiers.push(("language".to_string(), value.clone()));
                            } else if key == "language" && block_type == "code" {
                                // Handle explicit language attribute
                                modifiers.push((key.clone(), value.clone()));
                            } else {
                                modifiers.push((key, value));
                            }
                        }
                    }
                    
                    // Create a new block
                    let mut block = Block::new(&final_block_type, block_name.as_deref(), "");
                    
                    // Add modifiers
                    for (key, value) in modifiers {
                        block.add_modifier(&key, &value);
                    }
                    
                    // Push to the stack
                    block_stack.push(block);
                    content_stack.push(String::new());
                }
            },
            Ok(Event::End(ref e)) => {
                let name = str::from_utf8(e.name().as_ref())
                    .unwrap_or_default()
                    .to_string();
                
                // Handle document end tag (with or without meta: prefix)
                if name == "meta:document" || name == "document" {
                    in_document = false;
                    continue;
                }
                
                // Get the block type by removing meta: prefix if present
                let block_type = if name.starts_with("meta:") {
                    name.trim_start_matches("meta:").to_string()
                } else {
                    name
                };
                
                // Process end of any valid block type
                if in_document && is_valid_block_type(&block_type) {
                    if !block_stack.is_empty() {
                        // Pop the current block and its content
                        let mut block = block_stack.pop().unwrap();
                        let content = content_stack.pop().unwrap();
                        block.content = content.trim().to_string();
                        
                        // If there's a parent block, add this as a child
                        if !block_stack.is_empty() {
                            let parent_index = block_stack.len() - 1;
                            block_stack[parent_index].children.push(block);
                        } else {
                            // This is a top-level block
                            blocks.push(block);
                        }
                    }
                }
            },
            Ok(Event::Text(e)) => {
                if in_document && !block_stack.is_empty() {
                    if let Ok(text) = e.unescape() {
                        let last_idx = content_stack.len() - 1;
                        content_stack[last_idx].push_str(&text);
                    }
                }
            },
            Ok(Event::CData(e)) => {
                if in_document && !block_stack.is_empty() {
                    let text = str::from_utf8(e.as_ref())
                        .unwrap_or_default();
                    let last_idx = content_stack.len() - 1;
                    content_stack[last_idx].push_str(text);
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ParserError::ParseError(format!("XML parsing error: {}", e)));
            },
            _ => (),
        }
        
        buf.clear();
    }
    
    if blocks.is_empty() {
        return Err(ParserError::ParseError("No valid blocks found in XML document".to_string()));
    }
    
    // Debug output of parsed blocks
    println!("DEBUG: Parsed {} blocks from XML document", blocks.len());
    for (i, block) in blocks.iter().enumerate() {
        println!("DEBUG:   Block {}: type={}, name={:?}, children={}", 
                 i, block.block_type, block.name, block.children.len());
    }
    
    Ok(blocks)
}

/// Detect if a string is likely an XML document
pub fn is_xml_document(input: &str) -> bool {
    let trimmed = input.trim();
    
    // Check for XML declaration
    if trimmed.starts_with("<?xml") {
        return true;
    }
    
    // Check for root element with meta namespace
    if trimmed.contains("<meta:document") || 
       trimmed.contains("<document") ||
       trimmed.contains("xmlns:meta=") {
        return true;
    }
    
    // Check for common block types as XML tags
    let common_block_types = [
        "<code", "<data", "<shell", "<visualization", "<template", 
        "<variable", "<secret", "<filename", "<memory", "<api", 
        "<question", "<response", "<results", "<error_results", 
        "<error", "<preview", "<conditional", "<section"
    ];
    
    for block_type in common_block_types {
        if trimmed.contains(block_type) {
            return true;
        }
    }
    
    false
}
