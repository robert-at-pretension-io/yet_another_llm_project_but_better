use crate::parser::{Rule, extract_modifiers, extract_name};
use crate::parser::blocks::Block;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_response_block() {
        // Create a response block directly
        let mut block = Block::new("response", None, "The three laws of robotics, as defined by Isaac Asimov, are:
1. A robot may not injure a human being or, through inaction, allow a human being to come to harm.
2. A robot must obey the orders given it by human beings except where such orders would conflict with the First Law.
3. A robot must protect its own existence as long as such protection does not conflict with the First or Second Law.");
        
        // Add modifiers
        block.add_modifier("timestamp", "2023-05-15T14:30:00Z");
        block.add_modifier("tokens", "150");
        
        // Verify the block properties
        assert_eq!(block.block_type, "response");
        assert_eq!(block.content.lines().count(), 4);
        
        let timestamp = block.get_modifier("timestamp");
        assert_eq!(timestamp, Some(&"2023-05-15T14:30:00Z".to_string()));
    }
}

// Process question blocks
pub fn process_question_block(pair: pest::iterators::Pair<Rule>) -> Block {
    let mut block = Block::new("question", None, "");
    
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
