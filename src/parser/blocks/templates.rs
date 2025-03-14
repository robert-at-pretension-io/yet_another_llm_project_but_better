use crate::parser::{Rule, extract_name, extract_modifiers};
use crate::parser::blocks::Block;

// Process template blocks
pub fn process_template_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("template", None, "");
    let mut has_requires_modifier = false;
    let mut template_type = String::from("template");
    
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
                let modifiers = extract_modifiers(inner_pair);
                println!("DEBUG: Extracted {} modifiers for template", modifiers.len());
                
                for modifier in modifiers {
                    println!("DEBUG: Template modifier: '{}' = '{}'", modifier.0, modifier.1);
                    
                    if modifier.0 == "requires" {
                        has_requires_modifier = true;
                    }
                    if modifier.0 == "_type" {
                        template_type = format!("template:{}", modifier.1);
                        // Update block type immediately when _type modifier is found
                        block.block_type = template_type.clone();
                        println!("DEBUG: Updated template type to: {}", template_type);
                    }
                    
                    // Ensure we're properly adding all modifiers to the block
                    block.add_modifier(&modifier.0, &modifier.1);
                }
                
                // Debug: Print all modifiers in the block
                println!("DEBUG: Template block now has {} modifiers:", block.modifiers.len());
                for (k, v) in &block.modifiers {
                    println!("DEBUG:   '{}' = '{}'", k, v);
                }
            }
            Rule::block_content => {
                let content = inner_pair.as_str().trim().to_string();
                block.content = content.clone();
                
                // If content references api-call and no requires modifier exists, add it
                if content.contains("${api-call}") && !has_requires_modifier {
                    block.add_modifier("requires", "api-call");
                }
            }
            _ => {}
        }
    }
    
    block
}

// Process template invocation blocks
pub fn process_template_invocation(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("template_invocation", None, "");
    let mut invocation_type = String::from("template_invocation");
    let mut template_name = String::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::template_invocation_open => {
                for part in inner_pair.into_inner() {
                    if part.as_rule() == Rule::template_name {
                        template_name = part.as_str().to_string();
                        // Use a different naming scheme for invocations to avoid name collisions
                        // with the template definition
                        block.name = Some(format!("invoke-{}", template_name));
                        // Store the original template name as a modifier
                        block.add_modifier("template", &template_name);
                    } else if part.as_rule() == Rule::modifiers {
                        for modifier in extract_modifiers(part) {
                            if modifier.0 == "_type" {
                                invocation_type = format!("template_invocation:{}", modifier.1);
                            }
                            
                            // Ensure we're properly adding all modifiers to the block
                            block.add_modifier(&modifier.0, &modifier.1);
                        }
                        
                        // Update block type if _type modifier was found
                        if invocation_type != "template_invocation" {
                            block.block_type = invocation_type.clone();
                        }
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
