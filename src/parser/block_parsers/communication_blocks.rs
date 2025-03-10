use pest::Parser;
use crate::parser::{MetaLanguageParser, Rule, ParserError};
use crate::parser::blocks::Block;
use crate::parser::utils::extractors::{extract_name, extract_modifiers};

pub fn parse_question_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[question");
    let close_tag_pos = input.rfind("[/question]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid question block format".to_string()))?;
        
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
        let mut block = Block::new("question", name.as_deref(), content);
        
        // Extract model if present
        let model_start = input[open_tag_start..close_bracket].find("model:");
        if let Some(model_pos) = model_start {
            let model_pos = open_tag_start + model_pos + 6; // +6 to skip "model:"
            let model_end = input[model_pos..close_bracket].find(" ")
                .map(|pos| model_pos + pos)
                .unwrap_or(close_bracket);
            
            let model = input[model_pos..model_end].trim();
            block.add_modifier("model", model);
        }
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::question_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("question", None, "");
    
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

pub fn parse_response_block(input: &str) -> Result<Block, ParserError> {
    // Find the opening and closing tags
    let open_tag_start = input.find("[response");
    let close_tag_pos = input.rfind("[/response]");
    
    if let (Some(open_tag_start), Some(close_tag_pos)) = (open_tag_start, close_tag_pos) {
        let close_bracket = input[open_tag_start..].find("]")
            .map(|pos| open_tag_start + pos)
            .ok_or_else(|| ParserError::ParseError("Invalid response block format".to_string()))?;
        
        let mut block = Block::new("response", None, "");
        
        // Extract timestamp if present
        let timestamp_start = input[open_tag_start..close_bracket].find("timestamp:");
        if let Some(timestamp_pos) = timestamp_start {
            let timestamp_pos = open_tag_start + timestamp_pos + 10; // +10 to skip "timestamp:"
            
            // Handle quoted timestamp
            if input.chars().nth(timestamp_pos) == Some('"') {
                let timestamp_end = input[timestamp_pos + 1..close_bracket].find('"')
                    .map(|pos| timestamp_pos + 1 + pos)
                    .unwrap_or(close_bracket);
                
                let timestamp = input[timestamp_pos + 1..timestamp_end].trim();
                block.add_modifier("timestamp", timestamp);
            } else {
                let timestamp_end = input[timestamp_pos..close_bracket].find(" ")
                    .map(|pos| timestamp_pos + pos)
                    .unwrap_or(close_bracket);
                
                let timestamp = input[timestamp_pos..timestamp_end].trim();
                block.add_modifier("timestamp", timestamp);
            }
        }
        
        // Extract tokens if present
        let tokens_start = input[open_tag_start..close_bracket].find("tokens:");
        if let Some(tokens_pos) = tokens_start {
            let tokens_pos = open_tag_start + tokens_pos + 7; // +7 to skip "tokens:"
            let tokens_end = input[tokens_pos..close_bracket].find(" ")
                .map(|pos| tokens_pos + pos)
                .unwrap_or(close_bracket);
            
            let tokens = input[tokens_pos..tokens_end].trim();
            block.add_modifier("tokens", tokens);
        }
        
        // Extract content
        let content = input[close_bracket + 1..close_tag_pos].trim();
        block.content = content.to_string();
        
        return Ok(block);
    }
    
    // Fallback to the pest parser
    let pairs = MetaLanguageParser::parse(Rule::response_block, input)
        .map_err(|e| ParserError::ParseError(e.to_string()))?;
    
    let mut block = Block::new("response", None, "");
    
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
