use pest::Parser;
use crate::parser::{MetaLanguageParser, Rule, ParserError};
use crate::parser::blocks::Block;
use crate::parser::utils::extractors::{extract_name, extract_modifiers};

pub fn parse_data_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[data");
    let close_tag_pos = input.rfind("[/data]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid data block format".to_string()))?;
        
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
        let mut block = Block::new("data", name.as_deref(), content);
        
        // Extract format if present
        let format_start = input[open_tag_start..close_bracket].find("format:");
        if let Some(format_pos) = format_start {
            let format_pos = open_tag_start + format_pos + 7; // +7 to skip "format:"
            let format_end = input[format_pos..close_bracket].find(" ")
                .map(|pos| format_pos + pos)
                .unwrap_or(close_bracket);
            
            let format = input[format_pos..format_end].trim();
            block.add_modifier("format", format);
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::data_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("data", None, "");
    
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

pub fn parse_variable_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[variable");
    let close_tag_pos = input.rfind("[/variable]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid variable block format".to_string()))?;
        
        // Extract name (required for variables)
        let name_start = input[open_tag_start..close_bracket].find("name:");
        let name = if let Some(name_pos) = name_start {
            let name_pos = open_tag_start + name_pos + 5; // +5 to skip "name:"
            let name_end = input[name_pos..close_bracket].find(" ")
                .map(|pos| name_pos + pos)
                .unwrap_or(close_bracket);
            
            input[name_pos..name_end].trim().to_string()
        } else {
            return Err(ParserError::ParseError("Variable block requires a name".to_string()));
        };
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("variable", Some(&name), content);
        
        // Extract other modifiers
        let modifiers_text = input[open_tag_start + 9..close_bracket].trim(); // +9 to skip "[variable"
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
    let pairs = MetaLanguageParser::parse(Rule::variable_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("variable", None, "");
    
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
    
    if block.name.is_none() {
        return Err(ParserError::ParseError("Variable block requires a name".to_string()));
    }
    
    Ok(block)
}

pub fn parse_secret_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[secret");
    let close_tag_pos = input.rfind("[/secret]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid secret block format".to_string()))?;
        
        // Extract name (required for secrets)
        let name_start = input[open_tag_start..close_bracket].find("name:");
        let name = if let Some(name_pos) = name_start {
            let name_pos = open_tag_start + name_pos + 5; // +5 to skip "name:"
            let name_end = input[name_pos..close_bracket].find(" ")
                .map(|pos| name_pos + pos)
                .unwrap_or(close_bracket);
            
            input[name_pos..name_end].trim().to_string()
        } else {
            return Err(ParserError::ParseError("Secret block requires a name".to_string()));
        };
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("secret", Some(&name), content);
        
        // Extract env variable if present
        let env_start = input[open_tag_start..close_bracket].find("env:");
        if let Some(env_pos) = env_start {
            let env_pos = open_tag_start + env_pos + 4; // +4 to skip "env:"
            let env_end = input[env_pos..close_bracket].find(" ")
                .map(|pos| env_pos + pos)
                .unwrap_or(close_bracket);
            
            let env = input[env_pos..env_end].trim();
            block.add_modifier("env", env);
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::secret_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("secret", None, "");
    
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
    
    if block.name.is_none() {
        return Err(ParserError::ParseError("Secret block requires a name".to_string()));
    }
    
    Ok(block)
}

pub fn parse_memory_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[memory");
    let close_tag_pos = input.rfind("[/memory]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid memory block format".to_string()))?;
        
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
        let mut block = Block::new("memory", name.as_deref(), content);
        
        // Extract other modifiers
        let modifiers_text = input[open_tag_start + 7..close_bracket].trim(); // +7 to skip "[memory"
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
    let pairs = MetaLanguageParser::parse(Rule::memory_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("memory", None, "");
    
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

pub fn parse_filename_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[filename");
    let close_tag_pos = input.rfind("[/filename]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid filename block format".to_string()))?;
        
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
        let mut block = Block::new("filename", name.as_deref(), content);
        
        // Extract other modifiers
        let modifiers_text = input[open_tag_start + 9..close_bracket].trim(); // +9 to skip "[filename"
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
    let pairs = MetaLanguageParser::parse(Rule::filename_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("filename", None, "");
    
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
