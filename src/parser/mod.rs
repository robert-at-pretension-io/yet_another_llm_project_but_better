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
    
    // Add all blocks to top_level_blocks, including section blocks
    for block in blocks {
        top_level_blocks.push(block);
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
    
    // Try to identify the block type
    let block_type = if trimmed_content.starts_with("[code:") {
        // Extract language from [code:language]
        let lang_start = 6; // "[code:".len()
        // Make sure we don't go out of bounds
        if lang_start >= trimmed_content.len() {
            return None;
        }
        
        let lang_end = trimmed_content[lang_start..].find(' ')
            .or_else(|| trimmed_content[lang_start..].find(']'))
            .map(|pos| lang_start + pos)
            .unwrap_or(trimmed_content.len());
        
        let language = &trimmed_content[lang_start..lang_end];
        format!("code:{}", language)
    } else if trimmed_content.starts_with("[data") {
        "data".to_string()
    } else if trimmed_content.starts_with("[shell") {
        "shell".to_string()
    } else if trimmed_content.starts_with("[visualization") {
        "visualization".to_string()
    } else if trimmed_content.starts_with("[template") {
        "template".to_string()
    } else if trimmed_content.starts_with("[variable") {
        "variable".to_string()
    } else if trimmed_content.starts_with("[secret") {
        "secret".to_string()
    } else if trimmed_content.starts_with("[filename") {
        "filename".to_string()
    } else if trimmed_content.starts_with("[memory") {
        "memory".to_string()
    } else if trimmed_content.starts_with("[api") {
        "api".to_string()
    } else if trimmed_content.starts_with("[question") {
        "question".to_string()
    } else if trimmed_content.starts_with("[response") {
        "response".to_string()
    } else if trimmed_content.starts_with("[results") {
        "results".to_string()
    } else if trimmed_content.starts_with("[error_results") {
        "error_results".to_string()
    } else if trimmed_content.starts_with("[error") && !trimmed_content.starts_with("[error_results") {
        "error".to_string()
    } else if trimmed_content.starts_with("[preview") {
        "preview".to_string()
    } else if trimmed_content.starts_with("[conditional") {
        "conditional".to_string()
    } else {
        // Try to extract using the existing function
        extract_base_block_type(content)?
    };
    
    // Try to parse the block
    if let Ok(mut block) = block_parser::parse_single_block(content) {
        // If the block type wasn't properly set, set it now
        if block.block_type.is_empty() {
            block.block_type = block_type.clone();
        }
        
        // Find the end position of the block
        if let Some(end_pos) = find_block_end(content, &block_type) {
            return Some((block, end_pos));
        }
    }
    
    // Manual fallback parsing for common block types
    let open_bracket = trimmed_content.find('[')?;
    // Make sure we don't go out of bounds
    if open_bracket >= trimmed_content.len() {
        return None;
    }
    
    let close_bracket = trimmed_content[open_bracket..].find(']')
        .map(|pos| open_bracket + pos)?;
    
    // Validate that close_bracket is within bounds
    if close_bracket >= trimmed_content.len() {
        return None;
    }
    
    // Extract the opening tag
    let opening_tag = &trimmed_content[open_bracket..=close_bracket];
    
    // Extract name if present
    let mut name = None;
    if let Some(name_pos) = opening_tag.find("name:") {
        let name_start = name_pos + 5;
        let name_end = opening_tag[name_start..].find(' ')
            .map(|pos| name_start + pos)
            .unwrap_or_else(|| opening_tag[name_start..].find(']')
                .map(|pos| name_start + pos)
                .unwrap_or(opening_tag.len()));
        
        name = Some(opening_tag[name_start..name_end].to_string());
    }
    
    // Find the closing tag
    let closing_tag = format!("[/{}", block_type);
    
    // Make sure close_bracket + 1 doesn't overflow
    if close_bracket >= trimmed_content.len() {
        return None;
    }
    
    let content_start = close_bracket + 1;
    
    // Make sure content_start is within bounds
    if content_start >= trimmed_content.len() {
        return None;
    }
    
    if let Some(closing_start) = trimmed_content[content_start..].find(&closing_tag) {
        let closing_start = content_start + closing_start;
        
        // Make sure closing_start is within bounds
        if closing_start >= trimmed_content.len() {
            return None;
        }
        
        let closing_end = trimmed_content[closing_start..].find(']')
            .map(|pos| closing_start + pos + 1)?;
        
        // Make sure closing_end is within bounds
        if closing_end > trimmed_content.len() {
            return None;
        }
        
        // Extract content
        let content = trimmed_content[content_start..closing_start].trim();
        
        // Create the block
        let mut block = Block::new(&block_type, name.as_deref(), content);
        
        // Extract modifiers from opening tag
        if let Some(space_pos) = opening_tag.find(' ') {
            let modifiers_text = &opening_tag[space_pos..];
            // Here we could use a more sophisticated modifier extraction
            // For now, just look for name: which we already handled
        }
        
        // Adjust for whitespace, but be careful with subtraction
        let whitespace_offset = if content.len() > trimmed_content.len() {
            0 // This shouldn't happen, but prevents overflow
        } else {
            trimmed_content.len() - content.len()
        };
        
        return Some((block, closing_end + whitespace_offset));
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
    
    // Special case for code blocks which might have language in closing tag or not
    if block_type.starts_with("code:") {
        // Try with just [/code]
        let simple_close_tag = "[/code]";
        if let Some(close_pos) = content.find(simple_close_tag) {
            return Some(close_pos + simple_close_tag.len());
        }
    }
    
    // Handle other special cases
    match base_type {
        "data" => {
            let close_tag = "[/data]";
            if let Some(close_pos) = content.find(close_tag) {
                return Some(close_pos + close_tag.len());
            }
        },
        "shell" => {
            let close_tag = "[/shell]";
            if let Some(close_pos) = content.find(close_tag) {
                return Some(close_pos + close_tag.len());
            }
        },
        "visualization" => {
            let close_tag = "[/visualization]";
            if let Some(close_pos) = content.find(close_tag) {
                return Some(close_pos + close_tag.len());
            }
        },
        _ => {}
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
    let mut block = Block::new(&block_type, name.as_deref(), section_content);
    
    // Extract other modifiers (could be expanded)
    let opening_tag = &trimmed_content[start..open_end];
    if let Some(modifiers_pos) = opening_tag.find(' ') {
        let modifiers_text = &opening_tag[modifiers_pos..];
        // Use modifiers::extract_modifiers_from_text if we implement this later
        // For now, we'll just extract common modifiers manually
        
        // Example of extracting a specific modifier (can be expanded as needed)
        if let Some(format_pos) = modifiers_text.find("format:") {
            let format_start = format_pos + 7; // "format:".len()
            let format_end = modifiers_text[format_start..].find(' ')
                .map(|pos| format_start + pos)
                .unwrap_or_else(|| modifiers_text[format_start..].find(']')
                    .map(|pos| format_start + pos)
                    .unwrap_or(modifiers_text.len()));
            
            let format_value = modifiers_text[format_start..format_end].trim();
            block.add_modifier("format", format_value);
        }
    }
    
    // Parse child blocks from the content
    let mut remaining_content = section_content;
    
    // Use a more robust approach to find child blocks
    while !remaining_content.is_empty() {
        // Skip any text before the next block
        if let Some(block_start) = remaining_content.find('[') {
            // Make sure this is actually a block start and not just a bracket in text
            // Check if it's followed by a valid block type
            let potential_block = &remaining_content[block_start..];
            
            // Check if this is a valid block start by looking for common block types
            let is_valid_block_start = potential_block.starts_with("[code:") || 
                                      potential_block.starts_with("[data ") || 
                                      potential_block.starts_with("[data]") || 
                                      potential_block.starts_with("[shell ") || 
                                      potential_block.starts_with("[shell]") || 
                                      potential_block.starts_with("[visualization ") || 
                                      potential_block.starts_with("[visualization]") || 
                                      potential_block.starts_with("[section:") ||
                                      potential_block.starts_with("[template ") ||
                                      potential_block.starts_with("[template]") ||
                                      potential_block.starts_with("[variable ") ||
                                      potential_block.starts_with("[variable]") ||
                                      potential_block.starts_with("[secret ") ||
                                      potential_block.starts_with("[secret]") ||
                                      potential_block.starts_with("[filename ") ||
                                      potential_block.starts_with("[filename]") ||
                                      potential_block.starts_with("[memory ") ||
                                      potential_block.starts_with("[memory]") ||
                                      potential_block.starts_with("[api ") ||
                                      potential_block.starts_with("[api]") ||
                                      potential_block.starts_with("[question ") ||
                                      potential_block.starts_with("[question]") ||
                                      potential_block.starts_with("[response ") ||
                                      potential_block.starts_with("[response]") ||
                                      potential_block.starts_with("[results ") ||
                                      potential_block.starts_with("[results]") ||
                                      potential_block.starts_with("[error_results ") ||
                                      potential_block.starts_with("[error_results]") ||
                                      potential_block.starts_with("[error ") ||
                                      potential_block.starts_with("[error]") ||
                                      potential_block.starts_with("[preview ") ||
                                      potential_block.starts_with("[preview]") ||
                                      potential_block.starts_with("[conditional ") ||
                                      potential_block.starts_with("[conditional]");
            
            if is_valid_block_start {
                // Try to parse this as a block
                let block_content = &remaining_content[block_start..];
                
                // First check if it's a nested section
                if block_content.starts_with("[section:") {
                    if let Some((child_block, consumed)) = try_parse_section_block(block_content) {
                        block.add_child(child_block);
                        
                        // Move past this block
                        if block_start + consumed >= remaining_content.len() {
                            break;
                        }
                        remaining_content = &remaining_content[block_start + consumed..].trim_start();
                        continue;
                    }
                }
                
                // Try to parse as a regular block
                if let Some((child_block, consumed)) = try_parse_single_block(block_content) {
                    // Add the child block
                    block.add_child(child_block);
                    
                    // Move past this block
                    if block_start + consumed >= remaining_content.len() {
                        break;
                    }
                    remaining_content = &remaining_content[block_start + consumed..].trim_start();
                } else {
                    // Try a more direct approach for common block types
                    let mut parsed = false;
                    
                    // Handle data blocks
                    if block_content.starts_with("[data") {
                        if let Some(close_tag_pos) = block_content.find("[/data]") {
                            let close_end = close_tag_pos + 7; // "[/data]".len()
                            
                            // Extract name
                            let mut name = None;
                            if let Some(name_pos) = block_content[..close_tag_pos].find("name:") {
                                let name_start = name_pos + 5;
                                let name_end = block_content[name_start..close_tag_pos].find(' ')
                                    .map(|pos| name_start + pos)
                                    .unwrap_or_else(|| block_content[name_start..close_tag_pos].find(']')
                                        .map(|pos| name_start + pos)
                                        .unwrap_or(close_tag_pos));
                                
                                name = Some(block_content[name_start..name_end].trim().to_string());
                            }
                            
                            // Extract content
                            let open_end = block_content.find(']')? + 1;
                            let content = block_content[open_end..close_tag_pos].trim();
                            
                            // Create and add the block
                            let mut child_block = Block::new("data", name.as_deref(), content);
                            
                            // Extract format modifier if present
                            if let Some(format_pos) = block_content[..open_end].find("format:") {
                                let format_start = format_pos + 7;
                                let format_end = block_content[format_start..open_end].find(' ')
                                    .map(|pos| format_start + pos)
                                    .unwrap_or_else(|| block_content[format_start..open_end].find(']')
                                        .map(|pos| format_start + pos)
                                        .unwrap_or(open_end));
                                
                                let format = block_content[format_start..format_end].trim();
                                child_block.add_modifier("format", format);
                            }
                            
                            block.add_child(child_block);
                            
                            // Move past this block
                            if block_start + close_end >= remaining_content.len() {
                                break;
                            }
                            remaining_content = &remaining_content[block_start + close_end..].trim_start();
                            parsed = true;
                        }
                    }
                    // Handle code blocks
                    else if block_content.starts_with("[code:") {
                        // Extract language
                        let lang_start = 6; // "[code:".len()
                        let lang_end = block_content[lang_start..].find(' ')
                            .map(|pos| lang_start + pos)
                            .unwrap_or_else(|| block_content[lang_start..].find(']')
                                .map(|pos| lang_start + pos)
                                .unwrap_or(block_content.len()));
                        
                        let language = &block_content[lang_start..lang_end];
                        let block_type = format!("code:{}", language);
                        
                        // Find closing tag - try with language first, then without
                        let close_tag = format!("[/code:{}]", language);
                        let close_tag_pos = block_content.find(&close_tag)
                            .or_else(|| block_content.find("[/code]"));
                        
                        if let Some(close_tag_pos) = close_tag_pos {
                            let close_end = if block_content[close_tag_pos..].starts_with(&close_tag) {
                                close_tag_pos + close_tag.len()
                            } else {
                                close_tag_pos + 7 // "[/code]".len()
                            };
                            
                            // Extract name
                            let mut name = None;
                            if let Some(name_pos) = block_content[..close_tag_pos].find("name:") {
                                let name_start = name_pos + 5;
                                let name_end = block_content[name_start..close_tag_pos].find(' ')
                                    .map(|pos| name_start + pos)
                                    .unwrap_or_else(|| block_content[name_start..close_tag_pos].find(']')
                                        .map(|pos| name_start + pos)
                                        .unwrap_or(close_tag_pos));
                                
                                name = Some(block_content[name_start..name_end].trim().to_string());
                            }
                            
                            // Extract content
                            let open_end = block_content.find(']')? + 1;
                            let content = block_content[open_end..close_tag_pos].trim();
                            
                            // Create and add the block
                            let mut child_block = Block::new(&block_type, name.as_deref(), content);
                            
                            // Extract depends modifier if present
                            if let Some(depends_pos) = block_content[..open_end].find("depends:") {
                                let depends_start = depends_pos + 8;
                                let depends_end = block_content[depends_start..open_end].find(' ')
                                    .map(|pos| depends_start + pos)
                                    .unwrap_or_else(|| block_content[depends_start..open_end].find(']')
                                        .map(|pos| depends_start + pos)
                                        .unwrap_or(open_end));
                                
                                let depends = block_content[depends_start..depends_end].trim();
                                child_block.add_modifier("depends", depends);
                            }
                            
                            block.add_child(child_block);
                            
                            // Move past this block
                            if block_start + close_end >= remaining_content.len() {
                                break;
                            }
                            remaining_content = &remaining_content[block_start + close_end..].trim_start();
                            parsed = true;
                        }
                    }
                    
                    // If we couldn't parse using direct methods either, move ahead one character
                    if !parsed {
                        if block_start + 1 >= remaining_content.len() {
                            break;
                        }
                        remaining_content = &remaining_content[block_start + 1..];
                    }
                }
            } else {
                // Not a valid block start, just a bracket in text
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
    
    // Return with adjusted end position to account for leading whitespace
    Some((block, end_pos + whitespace_offset))
}
