use pest::Parser;
use crate::parser::{MetaLanguageParser, Rule, ParserError};
use crate::parser::blocks::Block;
use crate::parser::utils::extractors::{extract_name, extract_modifiers};

pub fn parse_section_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[section:");
    
    if let Some(open_tag_start) = open_tag_start {
        // Find section type
        let section_type_start = open_tag_start + 9; // +9 to skip "[section:"
        let section_type_end = input[section_type_start..].find(" ")
            .map(|pos| section_type_start + pos)
            .unwrap_or_else(|| input[section_type_start..].find("]")
                .map(|pos| section_type_start + pos)
                .unwrap_or(input.len()));
        
        let section_type = input[section_type_start..section_type_end].trim();
        
        // Find the end of opening tag
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid section block format".to_string()))?;
        
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
        let close_tag = format!("[/section:{}]", section_type);
        let close_tag_pos = input.rfind(&close_tag)
            .ok_or_else(|| ParserError::ParseError(format!("Missing closing tag: {}", close_tag)))?;
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new(&format!("section:{}", section_type), name.as_deref(), content);
        
        // Extract other modifiers
        let modifiers_text = input[section_type_end..close_bracket].trim();
        let mut parts = modifiers_text.split_whitespace();
        
        while let Some(part) = parts.next() {
            if part.starts_with("name:") {
                continue; // Already processed the name
            }
            
            if part.contains(":") {
                let key_value: Vec<&str> = part.split(":").collect();
                if key_value.len() >= 2 {
                    let key = key_value[0];
                    let mut value = key_value[1];
                    
                    // Handle quoted values
                    if value.starts_with("\"") && value.ends_with("\"") {
                        value = &value[1..value.len()-1];
                    }
                    
                    block.add_modifier(key, value);
                }
            }
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::section_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("section", None, "");
    let mut section_type = None;
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::section_type => {
                    section_type = Some(inner_pair.as_str().to_string());
                    block.block_type = format!("section:{}", inner_pair.as_str());
                }
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
    
    if section_type.is_none() {
        return Err(ParserError::ParseError("Section block requires a section type".to_string()));
    }
    
    Ok(block)
}

pub fn parse_conditional_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[conditional");
    let close_tag_pos = input.rfind("[/conditional]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid conditional block format".to_string()))?;
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("conditional", None, content);
        
        // Extract modifiers
        let modifiers_text = input[open_tag_start + 12..close_bracket].trim(); // +12 to skip "[conditional"
        let parts = modifiers_text.split_whitespace();
        
        for part in parts {
            if part.contains(":") {
                let key_value: Vec<&str> = part.split(":").collect();
                if key_value.len() >= 2 {
                    let key = key_value[0];
                    let mut value = key_value[1];
                    
                    // Handle quoted values
                    if value.starts_with("\"") && value.ends_with("\"") {
                        value = &value[1..value.len()-1];
                    }
                    
                    block.add_modifier(key, value);
                }
            }
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::conditional_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("conditional", None, "");
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
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

pub fn parse_error_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[error");
    let close_tag_pos = input.rfind("[/error]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid error block format".to_string()))?;
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("error", None, content);
        
        // Extract type if present
        let type_start = input[open_tag_start..close_bracket].find("type:");
        if let Some(type_pos) = type_start {
            let type_pos = open_tag_start + type_pos + 5; // +5 to skip "type:"
            let type_end = input[type_pos..close_bracket].find(" ")
                .map(|pos| type_pos + pos)
                .unwrap_or(close_bracket);
            
            let error_type = input[type_pos..type_end].trim();
            block.add_modifier("type", error_type);
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::error_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("error", None, "");
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
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

pub fn parse_visualization_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[visualization");
    let close_tag_pos = input.rfind("[/visualization]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid visualization block format".to_string()))?;
        
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
        let mut block = Block::new("visualization", name.as_deref(), content);
        
        // Extract other modifiers
        let modifiers_text = input[open_tag_start + 14..close_bracket].trim(); // +14 to skip "[visualization"
        for modifier in modifiers_text.split_whitespace() {
            if modifier.contains(":") && !modifier.starts_with("name:") {
                let parts: Vec<&str> = modifier.split(":").collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    block.add_modifier(key, &value.trim_matches('"'));
                }
            }
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::visualization_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("visualization", None, "");
    
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

pub fn parse_preview_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[preview");
    let close_tag_pos = input.rfind("[/preview]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid preview block format".to_string()))?;
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("preview", None, content);
        
        // Extract modifiers
        let modifiers_text = input[open_tag_start + 8..close_bracket].trim(); // +8 to skip "[preview"
        for modifier in modifiers_text.split_whitespace() {
            if modifier.contains(":") {
                let parts: Vec<&str> = modifier.split(":").collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    block.add_modifier(key, &value.trim_matches('"'));
                }
            }
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::preview_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("preview", None, "");
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
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
