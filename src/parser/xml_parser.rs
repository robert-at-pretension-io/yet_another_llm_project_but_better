use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::str;
use regex::Regex;

use crate::parser::blocks::Block;
use crate::parser::ParserError;
use crate::parser::is_valid_block_type;

/// Process reference tags in text content
fn process_reference_tags(text: &str) -> String {
    println!("DEBUG: Processing reference tags in: {}", text);
    
    // Use regex to find and process all reference tags - more flexible pattern to handle various formats
    let re = Regex::new(r#"<meta:reference\s+[^>]*target=["']([^"']+)["'][^>]*/?>"#);
    if let Err(e) = re {
        println!("ERROR: Invalid regex pattern: {}", e);
        return text.to_string();
    }
    let re = re.unwrap();
    
    let result = re.replace_all(text, |caps: &regex::Captures| {
        let target = &caps[1];
        println!("DEBUG: Found reference to: {}", target);
        
        // Create a special marker that will be recognized by the variable resolution code
        format!("__META_REFERENCE__{}", target)
    }).to_string();
    
    println!("DEBUG: After reference processing: {}", result);
    return result;
}

/// Parse an XML document into a vector of blocks
pub fn parse_xml_document(input: &str) -> Result<Vec<Block>, ParserError> {
    println!("DEBUG: Starting XML document parsing");
    println!("DEBUG: Input document length: {} characters", input.len());
    println!("DEBUG: First 100 chars: {}", &input[..std::cmp::min(100, input.len())]);
    
    let mut reader = Reader::from_str(input);
    reader.trim_text(true);
    
    let mut blocks = Vec::new();
    let mut buf = Vec::new();
    
    // Track if we're inside a document element (either meta:document or document)
    let mut in_document = false;
    
    // Track current block being built
    let mut block_stack: Vec<Block> = Vec::new();
    let mut content_stack: Vec<String> = Vec::new();
    
    println!("DEBUG: Beginning XML event loop");
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // Convert tag name to string - compatible with quick-xml 0.28
                let name_bytes = e.name().to_owned(); // Create owned copy
                let name = str::from_utf8(name_bytes.as_ref())
                    .unwrap_or_default()
                    .to_string();
                
                println!("DEBUG: Start tag: <{}>", name);
                
                // Check for document tag (with or without meta: prefix)
                if name == "meta:document" || name == "document" {
                    println!("DEBUG: Entering document element");
                    in_document = true;
                    continue;
                }
                
                // Only process elements inside document
                if !in_document {
                    println!("DEBUG: Skipping tag <{}> - not inside document element", name);
                    continue;
                }
                
                // Get the block type by removing meta: prefix if present
                let block_type = if name.starts_with("meta:") {
                    name.trim_start_matches("meta:").to_string()
                } else {
                    name
                };
                
                println!("DEBUG: Processing block type: {}", block_type);
                
                // Check if this is a valid block type
                if is_valid_block_type(&block_type) {
                    println!("DEBUG: Valid block type: {}", block_type);
                    
                    // Extract attributes
                    let mut block_name = None;
                    let mut modifiers = Vec::new();
                    let final_block_type = block_type.clone();
                    
                    println!("DEBUG: Extracting attributes for block type: {}", block_type);
                    // First check for special attribute formats in the raw tag
                    let name_bytes_raw = e.name().to_owned(); // Create owned copy
                    let raw_tag = str::from_utf8(name_bytes_raw.as_ref())
                        .unwrap_or_default()
                        .to_string();
                    
                    // We can't use e.raw() as it doesn't exist in this version of quick-xml
                    // Instead, we'll rely on the attribute parsing below
                    println!("DEBUG:   Processing attributes for tag: {}", raw_tag);
                    
                    // Process all attributes
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let raw_key = str::from_utf8(attr.key.as_ref())
                                .unwrap_or_default()
                                .to_string();
                            let value = str::from_utf8(&attr.value)
                                .unwrap_or_default()
                                .to_string();
                                
                            // Check if this is a name attribute in the format name="value"
                            if raw_key == "name" && !value.is_empty() {
                                println!("DEBUG:   Found name attribute with value: {}", value);
                                block_name = Some(value.clone());
                                continue;
                            }
                            
                            // Handle special case for attributes with format "name:value"
                            // This is now our primary way to detect name:value format
                            let (key, actual_value) = if raw_key.contains(':') {
                                let parts: Vec<&str> = raw_key.splitn(2, ':').collect();
                                // If this is a name:value attribute, set the block name
                                if parts[0] == "name" {
                                    println!("DEBUG:   Found name:value attribute: name:{}", parts[1]);
                                    block_name = Some(parts[1].to_string());
                                }
                                (parts[0].to_string(), parts[1].to_string())
                            } else {
                                (raw_key, value.clone())
                            };
                            
                            println!("DEBUG:   Attribute: {}=\"{}\"", key, actual_value);
                            
                            // Special debug for auto_execute and question blocks
                            if key == "auto_execute" {
                                println!("DEBUG:   Found auto_execute attribute with value: {}", actual_value);
                                println!("DEBUG:   For block type: {}", block_type);
                            }
                            
                            if key == "name" {
                                println!("DEBUG:   Setting block name to: {}", actual_value);
                                block_name = Some(actual_value);
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
                    
                    if final_block_type == "reference" {
                        println!("DEBUG: Processing reference block");
                        let target = modifiers.iter().find(|(k, _)| k == "target")
                            .map(|(_, v)| v.to_string());
                        if target.is_none() {
                            println!("ERROR: Reference block is missing required 'target' attribute");
                            return Err(ParserError::InvalidBlockStructure(
                                "Reference block is missing required 'target' attribute".to_string()
                            ));
                        }
                        let target_val = target.unwrap();
                        let mut block = Block::new(&final_block_type, block_name.as_deref(), "");
                        block.add_modifier("target", &target_val);
                        for (k, v) in modifiers.iter() {
                            if k != "name" && k != "target" {
                                block.add_modifier(k, v);
                            }
                        }
                        println!("DEBUG: Created reference block with target: {}", target_val);
                        block_stack.push(block);
                        content_stack.push(String::new());
                        println!("DEBUG: Pushed reference block to stack, stack size: {}", block_stack.len());
                        continue;
                    }
                    // Validate that block has a name attribute (now required)
                    if block_name.is_none() {
                        println!("ERROR: Block of type '{}' is missing required name attribute", final_block_type);
                        return Err(ParserError::InvalidBlockStructure(
                            format!("Block of type '{}' is missing required name attribute", final_block_type)
                        ));
                    }
                    
                    // Validate that response and results blocks have a "for" attribute
                    if (final_block_type == "response" || final_block_type == "results" || 
                        final_block_type == "error_results" || final_block_type == "error-response") && 
                       !modifiers.iter().any(|(k, _)| k == "for") {
                        println!("ERROR: {} block is missing required 'for' attribute", final_block_type);
                        return Err(ParserError::InvalidBlockStructure(
                            format!("{} block is missing required 'for' attribute", final_block_type)
                        ));
                    }
                    
                    // Create a new block
                    let mut block = Block::new(&final_block_type, block_name.as_deref(), "");
                    
                    println!("DEBUG: Created new block: type={}, name={:?}", 
                             final_block_type, block_name);
                    
                    // Double check that name was properly set
                    if let Some(name) = &block_name {
                        println!("DEBUG:   Block name confirmed: {}", name);
                        // Ensure the name is set in the block
                        block.name = Some(name.clone());
                    }
                    
                    // Special debug for question blocks
                    if final_block_type == "question" {
                        println!("DEBUG:   Created question block with name: {:?}", block_name);
                    }
                    
                    // Add modifiers
                    for (key, value) in modifiers {
                        block.add_modifier(&key, &value);
                        println!("DEBUG:   Added modifier: {}=\"{}\"", key, value);
                        
                        // Special debug for auto_execute modifier
                        if key == "auto_execute" {
                            println!("DEBUG:   Added auto_execute modifier with value: {} to block: {:?}", 
                                    value, block_name);
                            println!("DEBUG:   Block type: {}", final_block_type);
                        }
                    }
                    
                    // Push to the stack
                    block_stack.push(block);
                    content_stack.push(String::new());
                    println!("DEBUG: Pushed block to stack, stack size: {}", block_stack.len());
                }
            },
            Ok(Event::End(ref e)) => {
                let name_bytes = e.name().to_owned(); // Create owned copy
                let name = str::from_utf8(name_bytes.as_ref())
                    .unwrap_or_default()
                    .to_string();
                
                println!("DEBUG: End tag: </{}>", name);
                
                // Handle document end tag (with or without meta: prefix)
                if name == "meta:document" || name == "document" {
                    println!("DEBUG: Exiting document element");
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
                    println!("DEBUG: Processing end of block: {}", block_type);
                    if !block_stack.is_empty() {
                        // Pop the current block and its content
                        let mut block = block_stack.pop().unwrap();
                        let content = content_stack.pop().unwrap();
                        block.content = content.trim().to_string();
                        
                        println!("DEBUG: Block content length: {} characters", block.content.len());
                        println!("DEBUG: Content preview: {}", 
                                 &block.content[..std::cmp::min(50, block.content.len())]);
                        
                        // If there's a parent block, add this as a child
                        if !block_stack.is_empty() {
                            let parent_index = block_stack.len() - 1;
                            println!("DEBUG: Adding block as child to parent at index {}", parent_index);
                            
                            // Set the parent reference in the child block
                            if let Some(parent_name) = &block_stack[parent_index].name {
                                // Set parent reference in the child block
                                if let Some(child_name) = &block.name {
                                    println!("DEBUG: Setting parent of '{}' to '{}'", child_name, parent_name);
                                    block.parent = Some(parent_name.clone());
                                }
                            }
                            
                            block_stack[parent_index].children.push(block);
                        } else {
                            // This is a top-level block
                            println!("DEBUG: Adding block as top-level block");
                            blocks.push(block);
                        }
                    }
                }
            },
            Ok(Event::Text(e)) => {
                if in_document && !block_stack.is_empty() {
                    if let Ok(text) = e.unescape() {
                        println!("DEBUG: Text event: \"{}\"", text);
                        
                        // Check if the text contains a reference tag
                        if text.contains("<meta:reference") {
                            // Process reference tags in the text
                            let processed_text = process_reference_tags(&text);
                            println!("DEBUG: Processed reference tags in text: \"{}\"", processed_text);
                            
                            let last_idx = content_stack.len() - 1;
                            content_stack[last_idx].push_str(&processed_text);
                        } else {
                            let last_idx = content_stack.len() - 1;
                            content_stack[last_idx].push_str(&text);
                        }
                    }
                }
            },
            Ok(Event::CData(e)) => {
                if in_document && !block_stack.is_empty() {
                    let text = str::from_utf8(e.as_ref())
                        .unwrap_or_default();
                    println!("DEBUG: CDATA event, length: {} characters", text.len());

                    let last_idx = content_stack.len() - 1;
                    content_stack[last_idx].push_str(text);
                }
            },
            Ok(Event::Eof) => {
                println!("DEBUG: Reached end of XML document");
                break;
            },
            Err(e) => {
                println!("DEBUG: XML parsing error: {}", e);
                return Err(ParserError::ParseError(format!("XML parsing error: {}", e)));
            },
            _ => {
                println!("DEBUG: Other XML event type");
            },
        }
        
        buf.clear();
    }
    
    if blocks.is_empty() {
        println!("DEBUG: No blocks found in XML document");
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
