use pest::Parser;
use crate::parser::{MetaLanguageParser, Rule, ParserError};
use crate::parser::blocks::Block;
use crate::parser::utils::extractors::{extract_name, extract_modifiers};

pub fn parse_template_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[template");
    let close_tag_pos = input.rfind("[/template]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid template block format".to_string()))?;
        
        // Extract name (required for templates)
        let name_start = input[open_tag_start..close_bracket].find("name:");
        let name = if let Some(name_pos) = name_start {
            let name_pos = open_tag_start + name_pos + 5; // +5 to skip "name:"
            let name_end = input[name_pos..close_bracket].find(" ")
                .map(|pos| name_pos + pos)
                .unwrap_or(close_bracket);
            
            input[name_pos..name_end].trim().to_string()
        } else {
            return Err(ParserError::ParseError("Template block requires a name".to_string()));
        };
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("template", Some(&name), content);
        
        // Extract other modifiers
        let modifiers_text = input[open_tag_start + 9..close_bracket].trim(); // +9 to skip "[template"
        for modifier in modifiers_text.split_whitespace() {
            if modifier.contains(":") && !modifier.starts_with("name:") {
                let parts: Vec<&str> = modifier.split(":").collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim();
                    let mut value = parts[1].trim();
                    
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
    let pairs = MetaLanguageParser::parse(Rule::template_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("template", None, "");
    
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
        return Err(ParserError::ParseError("Template block requires a name".to_string()));
    }
    
    Ok(block)
}

pub fn parse_template_invocation(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[@");
    
    if let Some(open_tag_start) = open_tag_start {
        // Extract template name
        let template_name_start = open_tag_start + 2; // +2 to skip "[@"
        let template_name_end = input[template_name_start..].find(" ")
            .map(|pos| template_name_start + pos)
            .unwrap_or_else(|| input[template_name_start..].find("]")
                .map(|pos| template_name_start + pos)
                .unwrap_or(input.len()));
        
        let template_name = input[template_name_start..template_name_end].trim();
        
        // Find the end of opening tag
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid template invocation format".to_string()))?;
        
        // Find the closing tag
        let close_tag = format!("[/@{}]", template_name);
        let close_tag_pos = input.rfind(&close_tag)
            .ok_or_else(|| ParserError::ParseError(format!("Missing closing tag: {}", close_tag)))?;
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        
        // Create block
        let mut block = Block::new("template_invocation", Some(template_name), content);
        
        // Extract modifiers
        let modifiers_text = input[template_name_end..close_bracket].trim();
        
        // Parse modifiers - they might be a bit different in template invocations
        for modifier in modifiers_text.split_whitespace() {
            if modifier.contains(":") {
                let parts: Vec<&str> = modifier.split(":").collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim();
                    let mut value = parts[1].trim();
                    
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
    let pairs = MetaLanguageParser::parse(Rule::template_invocation_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("template_invocation", None, "");
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::template_name => {
                    block.name = Some(inner_pair.as_str().to_string());
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
        return Err(ParserError::ParseError("Template invocation requires a template name".to_string()));
    }
    
    Ok(block)
}
