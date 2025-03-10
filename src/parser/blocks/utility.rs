use crate::parser::{Rule, extract_name, extract_modifiers};
use crate::parser::blocks::Block;

// Process error blocks
pub fn process_error_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("error", None, "");
    
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
    
    block
}

// Process visualization blocks
pub fn process_visualization_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("visualization", None, "");
    
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
    
    block
}

// Process preview blocks
pub fn process_preview_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("preview", None, "");
    
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
    
    block
}

// Process section blocks
pub fn process_section_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("section", None, "");
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::section_type => {
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
    
    block
}

// Process conditional blocks
pub fn process_conditional_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("conditional", None, "");
    
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
    
    block
}
