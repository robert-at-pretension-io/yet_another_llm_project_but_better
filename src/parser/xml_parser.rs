use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::BufRead;

use crate::parser::blocks::Block;
use crate::parser::ParserError;

/// Parse an XML document into a vector of blocks
pub fn parse_xml_document(input: &str) -> Result<Vec<Block>, ParserError> {
    let mut reader = Reader::from_str(input);
    reader.trim_text(true);
    
    let mut blocks = Vec::new();
    let mut buf = Vec::new();
    
    // Track if we're inside a meta:document element
    let mut in_document = false;
    
    // Track current block being built
    let mut current_block: Option<Block> = None;
    let mut current_content = String::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = reader.decoder().decode(e.name().as_ref()).unwrap_or_default();
                
                // Check for meta:document
                if name == "meta:document" {
                    in_document = true;
                    continue;
                }
                
                // Only process elements inside meta:document
                if !in_document {
                    continue;
                }
                
                // Handle meta: prefixed elements as blocks
                if name.starts_with("meta:") {
                    let block_type = name.trim_start_matches("meta:").to_string();
                    
                    // Extract attributes
                    let mut block_name = None;
                    let mut modifiers = Vec::new();
                    
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key = reader.decoder().decode(attr.key.as_ref()).unwrap_or_default();
                            let value = reader.decoder().decode(&attr.value).unwrap_or_default();
                            
                            if key == "name" {
                                block_name = Some(value.to_string());
                            } else {
                                modifiers.push((key.to_string(), value.to_string()));
                            }
                        }
                    }
                    
                    // Create a new block
                    let mut block = Block::new(&block_type, block_name.as_deref(), "");
                    
                    // Add modifiers
                    for (key, value) in modifiers {
                        block.add_modifier(&key, &value);
                    }
                    
                    current_block = Some(block);
                    current_content.clear();
                }
            },
            Ok(Event::End(ref e)) => {
                let name = reader.decoder().decode(e.name().as_ref()).unwrap_or_default();
                
                if name == "meta:document" {
                    in_document = false;
                    continue;
                }
                
                if in_document && name.starts_with("meta:") {
                    if let Some(mut block) = current_block.take() {
                        // Set the content
                        block.content = current_content.trim().to_string();
                        blocks.push(block);
                    }
                    current_content.clear();
                }
            },
            Ok(Event::Text(e)) => {
                if in_document && current_block.is_some() {
                    let text = reader.decoder().decode(e.as_ref()).unwrap_or_default();
                    current_content.push_str(&text);
                }
            },
            Ok(Event::CData(e)) => {
                if in_document && current_block.is_some() {
                    let text = reader.decoder().decode(e.as_ref()).unwrap_or_default();
                    current_content.push_str(&text);
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
       trimmed.contains("xmlns:meta=") {
        return true;
    }
    
    false
}
