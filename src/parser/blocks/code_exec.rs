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
    
    println!("DEBUG: Extracting modifiers from raw tag: '{}'", tag_text);
    
    // Skip the "[api" part
    if let Some(after_api) = tag_text.strip_prefix("[api") {
        let mut remaining = after_api.trim();
        println!("DEBUG: After stripping [api prefix: '{}'", remaining);
        
        // Process each modifier
        while !remaining.is_empty() && !remaining.starts_with(']') {
            // Skip leading spaces
            remaining = remaining.trim_start();
            println!("DEBUG: Processing remaining text: '{}'", remaining);
            
            // Check for name: attribute first
            if remaining.starts_with("name:") {
                let name_start = 5; // "name:".len()
                let name_end = remaining[name_start..]
                    .find(|c: char| c.is_whitespace() || c == ']')
                    .map_or(remaining.len(), |pos| name_start + pos);
                
                let name_value = &remaining[name_start..name_end];
                println!("DEBUG: Found name attribute: '{}'", name_value);
                modifiers.push(("name".to_string(), name_value.to_string()));
                
                remaining = &remaining[name_end..];
                println!("DEBUG: Remaining after name: '{}'", remaining);
                continue;
            }
            
            // Look for key:value pattern
            if let Some(colon_pos) = remaining.find(':') {
                let key = remaining[..colon_pos].trim();
                println!("DEBUG: Found key: '{}'", key);
                
                // Skip the colon
                remaining = &remaining[colon_pos + 1..];
                println!("DEBUG: After colon: '{}'", remaining);
                
                // Check if value is quoted
                if remaining.starts_with('"') {
                    println!("DEBUG: Found quoted value");
                    // Find the closing quote
                    if let Some(quote_end) = remaining[1..].find('"') {
                        let value = &remaining[1..quote_end+1];
                        println!("DEBUG: Extracted quoted value: '{}'", value);
                        modifiers.push((key.to_string(), value.to_string()));
                        
                        // Move past the closing quote
                        remaining = &remaining[quote_end + 2..];
                        println!("DEBUG: Remaining after quoted value: '{}'", remaining);
                    } else {
                        // Malformed quoted value, just take until next space
                        println!("DEBUG: Malformed quoted value, no closing quote");
                        let value_end = remaining.find(|c: char| c.is_whitespace() || c == ']')
                            .unwrap_or(remaining.len());
                        let value = &remaining[..value_end];
                        println!("DEBUG: Extracted malformed value: '{}'", value);
                        modifiers.push((key.to_string(), value.to_string()));
                        
                        remaining = &remaining[value_end..];
                        println!("DEBUG: Remaining after malformed value: '{}'", remaining);
                    }
                } else {
                    // Unquoted value - read until whitespace or closing bracket
                    println!("DEBUG: Found unquoted value");
                    let value_end = remaining.find(|c: char| c.is_whitespace() || c == ']')
                        .unwrap_or(remaining.len());
                    let value = &remaining[..value_end];
                    println!("DEBUG: Extracted unquoted value: '{}'", value);
                    modifiers.push((key.to_string(), value.to_string()));
                    
                    remaining = &remaining[value_end..];
                    println!("DEBUG: Remaining after unquoted value: '{}'", remaining);
                }
            } else {
                // No more colons, break out
                println!("DEBUG: No more colons found, breaking out");
                break;
            }
        }
    }
    
    println!("DEBUG: Extracted {} modifiers in total", modifiers.len());
    for (i, (key, value)) in modifiers.iter().enumerate() {
        println!("DEBUG: Modifier {}: '{}' = '{}'", i+1, key, value);
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
        println!("DEBUG: Found {} direct modifiers from tag", direct_modifiers.len());
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
    for inner_pair in pair.clone().into_inner() {
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
    
    // If we still don't have modifiers, try a more direct approach
    if block.modifiers.is_empty() {
        println!("DEBUG: No modifiers found, trying direct extraction from raw text");
        let raw_text = pair.as_str();
        
        // Extract URL if present
        if let Some(url_start) = raw_text.find("url:") {
            let url_start = url_start + 4; // Skip "url:"
            let url_value = if raw_text[url_start..].starts_with('"') {
                // Quoted URL
                if let Some(end_quote) = raw_text[url_start+1..].find('"') {
                    let url = &raw_text[url_start+1..url_start+1+end_quote];
                    println!("DEBUG: Found quoted URL: '{}'", url);
                    url
                } else {
                    // No end quote found
                    let url_end = raw_text[url_start..].find(|c: char| c.is_whitespace() || c == ']')
                        .map_or(raw_text.len() - url_start, |pos| pos);
                    let url = &raw_text[url_start..url_start+url_end];
                    println!("DEBUG: Found unquoted URL: '{}'", url);
                    url
                }
            } else {
                // Unquoted URL
                let url_end = raw_text[url_start..].find(|c: char| c.is_whitespace() || c == ']')
                    .map_or(raw_text.len() - url_start, |pos| pos);
                let url = &raw_text[url_start..url_start+url_end];
                println!("DEBUG: Found unquoted URL: '{}'", url);
                url
            };
            
            block.add_modifier("url", url_value);
        }
        
        // Extract method if present
        if let Some(method_start) = raw_text.find("method:") {
            let method_start = method_start + 7; // Skip "method:"
            let method_end = raw_text[method_start..].find(|c: char| c.is_whitespace() || c == ']')
                .map_or(raw_text.len() - method_start, |pos| pos);
            let method = &raw_text[method_start..method_start+method_end];
            println!("DEBUG: Found method: '{}'", method);
            block.add_modifier("method", method);
        }
        
        // Extract headers if present
        if let Some(headers_start) = raw_text.find("headers:") {
            let headers_start = headers_start + 8; // Skip "headers:"
            let headers_value = if raw_text[headers_start..].starts_with('"') {
                // Quoted headers
                if let Some(end_quote) = raw_text[headers_start+1..].find('"') {
                    let headers = &raw_text[headers_start+1..headers_start+1+end_quote];
                    println!("DEBUG: Found quoted headers: '{}'", headers);
                    headers
                } else {
                    // No end quote found
                    let headers_end = raw_text[headers_start..].find(|c: char| c.is_whitespace() || c == ']')
                        .map_or(raw_text.len() - headers_start, |pos| pos);
                    let headers = &raw_text[headers_start..headers_start+headers_end];
                    println!("DEBUG: Found unquoted headers: '{}'", headers);
                    headers
                }
            } else {
                // Unquoted headers
                let headers_end = raw_text[headers_start..].find(|c: char| c.is_whitespace() || c == ']')
                    .map_or(raw_text.len() - headers_start, |pos| pos);
                let headers = &raw_text[headers_start..headers_start+headers_end];
                println!("DEBUG: Found unquoted headers: '{}'", headers);
                headers
            };
            
            block.add_modifier("headers", headers_value);
        }
    }
    
    // Debug: Print all modifiers in the final block
    println!("DEBUG: Final block has {} modifiers", block.modifiers.len());
    for (key, value) in &block.modifiers {
        println!("DEBUG: Final API modifier: '{}' = '{}'", key, value);
    }
    
    block
}
