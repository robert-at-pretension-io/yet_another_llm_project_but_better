use crate::parser::{Rule, extract_name, extract_modifiers};
use crate::parser::blocks::Block;

// Process template blocks
pub fn process_template_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("template", None, "");
    
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

// Process template invocation blocks
pub fn process_template_invocation(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("template_invocation", None, "");
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::template_invocation_open => {
                for part in inner_pair.into_inner() {
                    if part.as_rule() == Rule::template_name {
                        block.name = Some(part.as_str().to_string());
                    } else if part.as_rule() == Rule::modifiers {
                        for modifier in extract_modifiers(part) {
                            block.add_modifier(&modifier.0, &modifier.1);
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
