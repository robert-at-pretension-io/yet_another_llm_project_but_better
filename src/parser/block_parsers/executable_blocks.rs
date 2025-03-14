use pest::Parser;
use crate::parser::{MetaLanguageParser, Rule, ParserError};
use crate::parser::blocks::Block;
use crate::parser::utils::extractors::{extract_name, extract_modifiers};

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
        
        // Extract name if present
        let name_start = input[open_tag_start..close_bracket].find("name:");
        let mut name = None;
        
        if let Some(name_pos) = name_start {
            let name_pos = open_tag_start + name_pos + 5; // +5 to skip "name:"
            let name_end = input[name_pos..close_bracket].find(" ")
                .map(|pos| name_pos + pos)
                .unwrap_or(close_bracket);
            
            name = Some(input[name_pos..name_end].trim().to_string());
        }
        
        // Find the closing tag
        let close_tag = format!("[/code:{}]", language);
        let close_tag_pos = input.rfind(&close_tag)
            .ok_or_else(|| ParserError::ParseError(format!("Missing closing tag: {}", close_tag)))?;
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new(&format!("code:{}", language), name.as_deref(), content);
        
        // Extract modifiers with proper handling of quoted values
        let modifiers_text = input[language_end..close_bracket].trim();
        
        // Process modifiers with quoted values
        let mut i = 0;
        while i < modifiers_text.len() {
            // Skip whitespace
            while i < modifiers_text.len() && modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() {
                break;
            }
            
            // Find the key
            let key_start = i;
            while i < modifiers_text.len() && modifiers_text[i..i+1] != ":" && !modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() || modifiers_text[i..i+1] != ":" {
                i += 1;
                continue;
            }
            
            let key = modifiers_text[key_start..i].trim();
            i += 1; // Skip the colon
            
            // Skip whitespace after colon
            while i < modifiers_text.len() && modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() {
                break;
            }
            
            // Check if value is quoted
            if modifiers_text[i..i+1] == "\"" {
                i += 1; // Skip opening quote
                let value_start = i;
                
                // Find closing quote
                while i < modifiers_text.len() && modifiers_text[i..i+1] != "\"" {
                    i += 1;
                }
                
                if i < modifiers_text.len() {
                    let value = modifiers_text[value_start..i].trim();
                    if key != "name" {
                        block.add_modifier(key, value);
                    }
                    i += 1; // Skip closing quote
                }
            } else {
                // Unquoted value
                let value_start = i;
                while i < modifiers_text.len() && !modifiers_text[i..i+1].trim().is_empty() {
                    i += 1;
                }
                
                let value = modifiers_text[value_start..i].trim();
                if key != "name" {
                    block.add_modifier(key, value);
                }
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
                    name = extract_name(inner_pair);
                }
                Rule::modifiers => {
                    modifiers = extract_modifiers(inner_pair);
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

pub fn parse_shell_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[shell");
    let close_tag_pos = input.rfind("[/shell]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid shell block format".to_string()))?;
        
        // Extract name if present
        let name_start = input[open_tag_start..close_bracket].find("name:");
        let mut name = None;
        
        if let Some(name_pos) = name_start {
            let name_pos = open_tag_start + name_pos + 5; // +5 to skip "name:"
            let name_end = input[name_pos..close_bracket].find(" ")
                .map(|pos| name_pos + pos)
                .unwrap_or(close_bracket);
            
            name = Some(input[name_pos..name_end].trim().to_string());
        }
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("shell", name.as_deref(), content);
        
        // Extract modifiers with proper handling of quoted values
        let modifiers_text = input[open_tag_start + 6..close_bracket].trim(); // +6 to skip "[shell"
        
        // Process modifiers with quoted values
        let mut i = 0;
        while i < modifiers_text.len() {
            // Skip whitespace
            while i < modifiers_text.len() && modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() {
                break;
            }
            
            // Find the key
            let key_start = i;
            while i < modifiers_text.len() && modifiers_text[i..i+1] != ":" && !modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() || modifiers_text[i..i+1] != ":" {
                i += 1;
                continue;
            }
            
            let key = modifiers_text[key_start..i].trim();
            i += 1; // Skip the colon
            
            // Skip whitespace after colon
            while i < modifiers_text.len() && modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() {
                break;
            }
            
            // Check if value is quoted
            if modifiers_text[i..i+1] == "\"" {
                i += 1; // Skip opening quote
                let value_start = i;
                
                // Find closing quote
                while i < modifiers_text.len() && modifiers_text[i..i+1] != "\"" {
                    i += 1;
                }
                
                if i < modifiers_text.len() {
                    let value = modifiers_text[value_start..i].trim();
                    if key != "name" {
                        block.add_modifier(key, value);
                    }
                    i += 1; // Skip closing quote
                }
            } else {
                // Unquoted value
                let value_start = i;
                while i < modifiers_text.len() && !modifiers_text[i..i+1].trim().is_empty() {
                    i += 1;
                }
                
                let value = modifiers_text[value_start..i].trim();
                if key != "name" {
                    block.add_modifier(key, value);
                }
            }
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::shell_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("shell", None, "");
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::name_attr => {
                    block.name = extract_name(inner_pair);
                }
                Rule::modifiers => {
                    for modifier in extract_modifiers(inner_pair) {
                        block.add_modifier(&modifier.0, &modifier.1);
                    }
                }
                Rule::block_content => {
                    block.content = inner_pair.as_str().trim().to_string();
                }
                _ => {}
            }
        }
    }
    
    Ok(block)
}

pub fn parse_api_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[api");
    let close_tag_pos = input.rfind("[/api]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid api block format".to_string()))?;
        
        // Extract name if present
        let name_start = input[open_tag_start..close_bracket].find("name:");
        let mut name = None;
        
        if let Some(name_pos) = name_start {
            let name_pos = open_tag_start + name_pos + 5; // +5 to skip "name:"
            let name_end = input[name_pos..close_bracket].find(" ")
                .map(|pos| name_pos + pos)
                .unwrap_or(close_bracket);
            
            name = Some(input[name_pos..name_end].trim().to_string());
        }
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("api", name.as_deref(), content);
        
        // Extract modifiers with proper handling of quoted values
        let modifiers_text = input[open_tag_start + 4..close_bracket].trim(); // +4 to skip "[api"
        
        // Process modifiers with quoted values
        let mut i = 0;
        while i < modifiers_text.len() {
            // Skip whitespace
            while i < modifiers_text.len() && modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() {
                break;
            }
            
            // Find the key
            let key_start = i;
            while i < modifiers_text.len() && modifiers_text[i..i+1] != ":" && !modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() || modifiers_text[i..i+1] != ":" {
                i += 1;
                continue;
            }
            
            let key = modifiers_text[key_start..i].trim();
            i += 1; // Skip the colon
            
            // Skip whitespace after colon
            while i < modifiers_text.len() && modifiers_text[i..i+1].trim().is_empty() {
                i += 1;
            }
            if i >= modifiers_text.len() {
                break;
            }
            
            // Check if value is quoted
            if modifiers_text[i..i+1] == "\"" {
                i += 1; // Skip opening quote
                let value_start = i;
                
                // Find closing quote
                while i < modifiers_text.len() && modifiers_text[i..i+1] != "\"" {
                    i += 1;
                }
                
                if i < modifiers_text.len() {
                    let value = modifiers_text[value_start..i].trim();
                    if key != "name" {
                        block.add_modifier(key, value);
                    }
                    i += 1; // Skip closing quote
                }
            } else {
                // Unquoted value
                let value_start = i;
                while i < modifiers_text.len() && !modifiers_text[i..i+1].trim().is_empty() {
                    i += 1;
                }
                
                let value = modifiers_text[value_start..i].trim();
                if key != "name" {
                    block.add_modifier(key, value);
                }
            }
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::api_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("api", None, "");
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::name_attr => {
                    block.name = extract_name(inner_pair);
                }
                Rule::modifiers => {
                    for modifier in extract_modifiers(inner_pair) {
                        block.add_modifier(&modifier.0, &modifier.1);
                    }
                }
                Rule::block_content => {
                    block.content = inner_pair.as_str().trim().to_string();
                }
                _ => {}
            }
        }
    }
    
    Ok(block)
}
