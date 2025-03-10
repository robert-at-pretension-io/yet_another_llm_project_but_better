use crate::parser::{Rule, extract_modifiers};
use crate::parser::blocks::Block;

// Process question blocks
pub fn process_question_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("question", None, "");
    
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

// Process response blocks
pub fn process_response_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("response", None, "");
    
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
