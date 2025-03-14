use crate::parser::Rule;
use crate::parser::blocks::Block;
use crate::parser::utils::extractors::extract_modifiers;

pub fn process_results_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("results", None, "");
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::modifiers => {
                for modifier in extract_modifiers(inner_pair) {
                    block.add_modifier(&modifier.0, &modifier.1);
                }
            }
            Rule::block_content => {
                block.content = inner_pair.as_str().to_string();
            }
            _ => {}
        }
    }
    
    block
}

pub fn process_error_results_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("error_results", None, "");
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::modifiers => {
                for modifier in extract_modifiers(inner_pair) {
                    block.add_modifier(&modifier.0, &modifier.1);
                }
            }
            Rule::block_content => {
                block.content = inner_pair.as_str().to_string();
            }
            _ => {}
        }
    }
    
    block
}
