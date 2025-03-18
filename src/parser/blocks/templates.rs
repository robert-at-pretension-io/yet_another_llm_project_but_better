use crate::parser::{Rule, extract_name, extract_modifiers};
use crate::parser::blocks::Block;

// Process template blocks
pub fn process_template_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("template", None, "");
    let mut has_requires_modifier = false;
    
    println!("DEBUG: Processing template block: '{}'", pair.as_str());
    
    for inner_pair in pair.into_inner() {
        println!("DEBUG: Template inner rule: {:?}", inner_pair.as_rule());
        
        match inner_pair.as_rule() {
            Rule::name_attr => {
                block.name = extract_name(inner_pair);
                println!("DEBUG: Template name: {:?}", block.name);
            }
            Rule::modifiers => {
                println!("DEBUG: Processing template modifiers: '{}'", inner_pair.as_str());
                
                // Extract modifiers and handle special cases
                let raw_modifiers_text = inner_pair.as_str().trim();
                println!("DEBUG: Raw modifiers text: '{}'", raw_modifiers_text);
                
                // Use the extractor to parse modifiers
                let modifiers = extract_modifiers(inner_pair);
                println!("DEBUG: Extracted {} modifiers for template", modifiers.len());
                
                // Process modifiers one by one
                for (key, value) in modifiers {
                    println!("DEBUG: Processing template modifier: '{}' = '{}'", key, value);
                    
                    // Special handling for certain modifiers
                    match key.as_str() {
                        "requires" => {
                            has_requires_modifier = true;
                            block.add_modifier(&key, &value);
                        },
                        "_type" => {
                            // Type modifier changes the block type
                            let template_type = format!("template:{}", value);
                            block.block_type = template_type.clone();
                            println!("DEBUG: Updated template type to: {}", template_type);
                            block.add_modifier(&key, &value);
                        },
                        _ => {
                            // Regular modifier handling
                            block.add_modifier(&key, &value);
                        }
                    }
                }
                
                // Debug: Print all modifiers in the block
                println!("DEBUG: Template block now has {} modifiers:", block.modifiers.len());
                for (k, v) in &block.modifiers {
                    println!("DEBUG:   '{}' = '{}'", k, v);
                }
            }
            Rule::block_content => {
                let content = inner_pair.as_str().trim().to_string();
                println!("DEBUG: Template content: '{}'", content);
                block.content = content.clone();
                
                // If content references api-call and no requires modifier exists, add it
                if content.contains("${api-call}") && !has_requires_modifier {
                    println!("DEBUG: Adding implicit api-call dependency");
                    block.add_modifier("requires", "api-call");
                }
            }
            _ => {
                println!("DEBUG: Ignoring unknown template part: {:?}", inner_pair.as_rule());
            }
        }
    }
    
    println!("DEBUG: Created template block: type={}, name={:?}, modifiers={}",
             block.block_type, block.name, block.modifiers.len());
    
    block
}

// Process template invocation blocks
pub fn process_template_invocation(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("template_invocation", None, "");
    let mut template_name = String::new();
    
    println!("DEBUG: Processing template invocation: '{}'", pair.as_str());
    
    for inner_pair in pair.into_inner() {
        println!("DEBUG: Invocation inner rule: {:?}", inner_pair.as_rule());
        
        match inner_pair.as_rule() {
            Rule::template_invocation_open => {
                for part in inner_pair.into_inner() {
                    println!("DEBUG: Invocation open part: {:?} - '{}'", part.as_rule(), part.as_str());
                    
                    if part.as_rule() == Rule::template_name {
                        template_name = part.as_str().to_string();
                        println!("DEBUG: Found template name: '{}'", template_name);
                        
                        // Store the original template name as a modifier for reference
                        block.add_modifier("template", &template_name);
                        
                        // Don't set the name yet - check for an explicit name in modifiers first
                    } else if part.as_rule() == Rule::modifiers {
                        let modifiers = extract_modifiers(part);
                        println!("DEBUG: Found {} invocation modifiers", modifiers.len());
                        
                        // Check for an explicit name in the modifiers
                        let explicit_name = modifiers.iter()
                            .find(|(k, _)| k == "name")
                            .map(|(_, v)| v.clone());
                        
                        // Set the name of the block - use the explicit name if provided
                        if let Some(name) = explicit_name {
                            println!("DEBUG: Using explicit name for invocation: '{}'", name);
                            block.name = Some(name);
                        } else {
                            // Otherwise use the template name as the block name
                            println!("DEBUG: Using template name for invocation: '{}'", template_name);
                            block.name = Some(template_name.clone());
                        }
                        
                        // Process remaining modifiers
                        for modifier in modifiers {
                            if modifier.0 == "_type" {
                                let invocation_type = format!("template_invocation:{}", modifier.1);
                                block.block_type = invocation_type.clone();
                            }
                            
                            // Add all modifiers except the name (already used for block.name)
                            if modifier.0 != "name" {
                                block.add_modifier(&modifier.0, &modifier.1);
                            }
                        }
                    }
                }
            }
            Rule::block_content => {
                let content = inner_pair.as_str().trim().to_string();
                println!("DEBUG: Invocation content: '{}'", content);
                block.content = content;
            }
            _ => {
                println!("DEBUG: Ignoring unknown invocation part: {:?}", inner_pair.as_rule());
            }
        }
    }
    
    // If no name was set, fallback to using the template name
    if block.name.is_none() && !template_name.is_empty() {
        println!("DEBUG: Setting fallback name for invocation: '{}'", template_name);
        block.name = Some(template_name);
    }
    
    println!("DEBUG: Created invocation block: type={}, name={:?}", 
             block.block_type, block.name);
    
    block
}
