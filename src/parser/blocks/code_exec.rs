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

// Process API blocks
pub fn process_api_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("api", None, "");
    
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
