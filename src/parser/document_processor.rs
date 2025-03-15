use crate::parser::blocks::Block;

/// Process a document to establish relationships between blocks
pub fn update_document(blocks: &mut Vec<Block>) {
    // First pass: Collect all question blocks and their names
    let mut question_blocks: Vec<(usize, Option<String>)> = Vec::new();
    
    for (i, block) in blocks.iter().enumerate() {
        if block.block_type == "question" {
            question_blocks.push((i, block.name.clone()));
        }
    }
    
    // Second pass: Link response blocks to their questions
    for i in 0..blocks.len() {
        if blocks[i].block_type == "response" {
            // If the response has a name, find the question with that name
            if let Some(name) = &blocks[i].name {
                for (q_idx, q_name) in &question_blocks {
                    if let Some(q_name) = q_name {
                        if q_name == name {
                            blocks[i].add_modifier("question_ref", &q_idx.to_string());
                            break;
                        }
                    }
                }
            } else {
                // For unnamed responses, find the nearest preceding unnamed question
                for (q_idx, q_name) in question_blocks.iter().rev() {
                    if q_name.is_none() && *q_idx < i {
                        blocks[i].add_modifier("question_ref", &q_idx.to_string());
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unnamed_question_response_pairing() {
        let mut blocks = vec![
            Block::new("question", None, "What is the meaning of life?"),
            Block::new("response", None, "42")
        ];
        
        update_document(&mut blocks);
        
        assert!(blocks[1].has_modifier("question_ref"));
        assert_eq!(blocks[1].get_modifier("question_ref").unwrap(), "0");
    }
    
    #[test]
    fn test_named_question_response_pairing() {
        let mut blocks = vec![
            {
                let mut block = Block::new("question", Some("q1"), "What is the meaning of life?");
                block
            },
            {
                let mut block = Block::new("response", Some("q1"), "42");
                block
            }
        ];
        
        update_document(&mut blocks);
        
        assert!(blocks[1].has_modifier("question_ref"));
        assert_eq!(blocks[1].get_modifier("question_ref").unwrap(), "0");
    }
}
