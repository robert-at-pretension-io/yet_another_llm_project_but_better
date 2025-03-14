use pest::Parser;
use crate::parser::{MetaLanguageParser, Rule, ParserError};
use crate::parser::blocks::Block;
use crate::parser::utils::extractors::{extract_name, extract_modifiers};
use crate::parser::modifiers::{extract_modifiers_from_text, extract_dependencies};

// Process code blocks
pub fn process_code_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut language = String::new();
    let mut name = None;
    let mut modifiers = Vec::new();
    let mut content = String::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::language => {
                language = inner_pair.as_str().to_string();
            }
            Rule::name_attr => {
                name = extract_name(inner_pair);
            }
            Rule::modifiers => {
                // Get the full text to extract dependencies
                let modifier_text = inner_pair.as_str();
                
                // Get dependencies from text
                let deps = extract_dependencies(modifier_text);
                modifiers.extend(deps);
                
                // Get other modifiers
                for m in extract_modifiers(inner_pair) {
                    if m.0 != "depends" && m.0 != "requires" {
                        modifiers.push(m);
                    }
                }
            }
            Rule::block_content => {
                content = inner_pair.as_str().trim().to_string();
            }
            _ => {}
        }
    }
    
    let mut block = Block::new(&format!("code:{}", language), name.as_deref(), &content);
    for (key, value) in modifiers {
        block.add_modifier(&key, &value);
    }
    
    block
}

pub fn parse_code_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[code:");
    
    if let Some(open_tag_start) = open_tag_start {
        // Extract language
        let language_start = open_tag_start + 6; // +6 to skip "[code:"
        let language_end = input[language_start..].find(" ")
            .map(|pos| language_start + pos)
            .unwrap_or_else(|| input[language_start..].find("]")
                .map(|pos| language_start + pos)
                .unwrap_or(input.len()));
        
        let language = input[language_start..language_end].trim();
        
        // Find the end of opening tag
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid code block format".to_string()))?;
        
        // Extract modifiers text
        let modifiers_text = input[language_end..close_bracket].trim();
        
        // Extract name if present
        let name_start = modifiers_text.find("name:");
        let mut name = None;
        
        if let Some(name_pos) = name_start {
            let name_pos = name_pos + 5; // +5 to skip "name:"
            let name_end = modifiers_text[name_pos..].find(" ")
                .map(|pos| name_pos + pos)
                .unwrap_or(modifiers_text.len());
            
            name = Some(modifiers_text[name_pos..name_end].trim().to_string());
        }
        
        // Find the closing tag
        let close_tag = format!("[/code:{}]", language);
        let close_tag_pos = input.rfind(&close_tag)
            .ok_or_else(|| ParserError::ParseError(format!("Missing closing tag: {}", close_tag)))?;
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new(&format!("code:{}", language), name.as_deref(), content);
        
        // Extract all modifiers
        let modifiers = extract_modifiers_from_text(modifiers_text);
        for (key, value) in modifiers {
            if key != "name" || name.is_none() {
                block.add_modifier(&key, &value);
            }
        }
        
        // Add any dependencies modifiers that might be complex
        let deps_modifiers = extract_dependencies(modifiers_text);
        for (key, value) in deps_modifiers {
            if !block.has_modifier(&key) {
                block.add_modifier(&key, &value);
            }
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::code_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut language = String::new();
    let mut name = None;
    let mut modifiers = Vec::new();
    let mut content = String::new();
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::language => {
                    language = inner_pair.as_str().to_string();
                }
                Rule::name_attr => {
                    for np in inner_pair.into_inner() {
                        if np.as_rule() == Rule::block_name {
                            name = Some(np.as_str().to_string());
                        }
                    }
                }
                Rule::modifiers => {
                    // Get the full text to extract dependencies
                    let modifier_text = inner_pair.as_str();
                    
                    // Get dependencies from text
                    let deps = extract_dependencies(modifier_text);
                    modifiers.extend(deps);
                    
                    // Process normal modifiers
                    for mod_pair in inner_pair.into_inner() {
                        if mod_pair.as_rule() == Rule::modifier {
                            let mut key = String::new();
                            let mut value = String::new();
                            
                            for part in mod_pair.into_inner() {
                                match part.as_rule() {
                                    Rule::modifier_key => {
                                        key = part.as_str().to_string();
                                    }
                                    Rule::modifier_value => {
                                        if let Some(val_part) = part.into_inner().next() {
                                            value = val_part.as_str().trim_matches('"').to_string();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            
                            if !key.is_empty() && key != "depends" && key != "requires" {
                                modifiers.push((key, value));
                            }
                        }
                    }
                }
                Rule::block_content => {
                    content = inner_pair.as_str().trim().to_string();
                }
                _ => {}
            }
        }
    }
    
    let mut block = Block::new(&format!("code:{}", language), name.as_deref(), &content);
    for (key, value) in modifiers {
        block.add_modifier(&key, &value);
    }
    
    Ok(block)
}

// Process shell blocks
pub fn process_shell_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("shell", None, "");
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::name_attr => {
                block.name = extract_name(inner_pair);
            }
            Rule::modifiers => {
                // Get full text for dependency extraction
                let modifier_text = inner_pair.as_str();
                
                // Extract dependencies
                let deps = extract_dependencies(modifier_text);
                for (key, value) in deps {
                    block.add_modifier(&key, &value);
                }
                
                // Extract other modifiers
                for modifier in extract_modifiers(inner_pair) {
                    if modifier.0 != "depends" && modifier.0 != "requires" {
                        block.add_modifier(&modifier.0, &modifier.1);
                    }
                }
            }
            Rule::block_content => {
                block.content = inner_pair.as_str().trim().to_string();
            }
            _ => {}
        }
    }
    
    block
}

// Helper function to extract modifiers directly from API block tag
fn extract_api_modifiers_from_tag(tag_text: &str) -> Vec<(String, String)> {
    let mut modifiers = Vec::new();
    
    // Skip the "[api" part
    if let Some(after_api) = tag_text.strip_prefix("[api") {
        let mut remaining = after_api.trim();
        
        // Process each modifier
        while !remaining.is_empty() && !remaining.starts_with(']') {
            // Skip leading spaces
            remaining = remaining.trim_start();
            
            // Check for name: attribute first
            if remaining.starts_with("name:") {
                let name_start = 5; // "name:".len()
                let name_end = remaining[name_start..]
                    .find(|c: char| c.is_whitespace() || c == ']')
                    .map_or(remaining.len(), |pos| name_start + pos);
                
                let name_value = &remaining[name_start..name_end];
                modifiers.push(("name".to_string(), name_value.to_string()));
                
                remaining = &remaining[name_end..];
                continue;
            }
            
            // Look for key:value pattern
            if let Some(colon_pos) = remaining.find(':') {
                let key = remaining[..colon_pos].trim();
                
                // Skip the colon
                remaining = &remaining[colon_pos + 1..];
                
                // Check if value is quoted
                if remaining.starts_with('"') {
                    // Find the closing quote
                    if let Some(quote_end) = remaining[1..].find('"') {
                        let value = &remaining[1..=quote_end];
                        modifiers.push((key.to_string(), value.to_string()));
                        
                        // Move past the closing quote
                        remaining = &remaining[quote_end + 2..];
                    } else {
                        // Malformed quoted value, just take until next space
                        let value_end = remaining.find(|c: char| c.is_whitespace() || c == ']')
                            .unwrap_or(remaining.len());
                        let value = &remaining[..value_end];
                        modifiers.push((key.to_string(), value.to_string()));
                        
                        remaining = &remaining[value_end..];
                    }
                } else {
                    // Unquoted value - read until whitespace or closing bracket
                    let value_end = remaining.find(|c: char| c.is_whitespace() || c == ']')
                        .unwrap_or(remaining.len());
                    let value = &remaining[..value_end];
                    modifiers.push((key.to_string(), value.to_string()));
                    
                    remaining = &remaining[value_end..];
                }
            } else {
                // No more colons, break out
                break;
            }
        }
    }
    
    modifiers
}

// Process API blocks
pub fn process_api_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("api", None, "");
    
    // Debug: Print the raw API block
    println!("DEBUG: Processing API block: '{}'", pair.as_str());
    
    // Direct extraction from the raw tag
    let raw_text = pair.as_str();
    if let Some(tag_end) = raw_text.find(']') {
        let opening_tag = &raw_text[..=tag_end];
        println!("DEBUG: API opening tag: '{}'", opening_tag);
        
        // Extract modifiers directly from the tag
        let direct_modifiers = extract_api_modifiers_from_tag(opening_tag);
        for (key, value) in &direct_modifiers {
            println!("DEBUG: Direct API modifier: '{}' = '{}'", key, value);
            if key == "name" {
                block.name = Some(value.clone());
            } else {
                block.add_modifier(key, value);
            }
        }
    }
    
    // Also process using the standard approach
    for inner_pair in pair.into_inner() {
        println!("DEBUG: API inner rule: {:?}", inner_pair.as_rule());
        
        match inner_pair.as_rule() {
            Rule::name_attr => {
                if block.name.is_none() {
                    block.name = extract_name(inner_pair);
                }
                println!("DEBUG: API block name: {:?}", block.name);
            }
            Rule::modifiers => {
                // Get full text for dependency extraction
                let modifier_text = inner_pair.as_str();
                println!("DEBUG: API modifiers text: '{}'", modifier_text);
                
                // Extract dependencies
                let deps = extract_dependencies(modifier_text);
                for (key, value) in &deps {
                    println!("DEBUG: Adding API dependency: '{}' = '{}'", key, value);
                    block.add_modifier(key, value);
                }
                
                // Extract other modifiers
                for modifier in extract_modifiers(inner_pair.clone()) {
                    if modifier.0 != "depends" && modifier.0 != "requires" && !block.has_modifier(&modifier.0) {
                        println!("DEBUG: Adding API modifier: '{}' = '{}'", modifier.0, modifier.1);
                        block.add_modifier(&modifier.0, &modifier.1);
                    }
                }
                
                // Ensure we're getting all modifiers by also using the text-based extraction
                // This is especially important for quoted values
                let text_modifiers = extract_modifiers_from_text(modifier_text);
                for (key, value) in text_modifiers {
                    if key != "name" && key != "depends" && key != "requires" && !block.has_modifier(&key) {
                        println!("DEBUG: Adding API text modifier: '{}' = '{}'", key, value);
                        block.add_modifier(&key, &value);
                    }
                }
            }
            Rule::block_content => {
                block.content = inner_pair.as_str().trim().to_string();
                println!("DEBUG: API block content: '{}'", block.content);
            }
            _ => {
                println!("DEBUG: Unknown API rule: {:?}", inner_pair.as_rule());
            }
        }
    }
    
    // Debug: Print all modifiers in the final block
    for (key, value) in &block.modifiers {
        println!("DEBUG: Final API modifier: '{}' = '{}'", key, value);
    }
    
    block
}
