#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    
    #[test]
    fn test_question_block() {
        // Create a question block directly
        let mut block = Block::new("question", Some("simple-question"), "What are the three laws of robotics?");
        block.add_modifier("model", "gpt-4");
        
        // Verify the block properties
        assert_eq!(block.block_type, "question");
        assert_eq!(block.name, Some("simple-question".to_string()));
        assert_eq!(block.content, "What are the three laws of robotics?");
        
        let model = block.get_modifier("model");
        assert_eq!(model, Some(&"gpt-4".to_string()));
    }
    
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
    
    #[test]
    fn test_question_response_sequence() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question name="robotics-laws">
  What are the three laws of robotics?
  </meta:question>

  <meta:response name="robotics-laws-response" for="robotics-laws">
  The three laws of robotics are:
  1. A robot may not harm a human.
  2. A robot must obey human orders.
  3. A robot must protect its own existence.
  </meta:response>
</meta:document>"#;
        
        let result = parse_document(input).unwrap();
        assert_eq!(result.len(), 2);
        
        assert_eq!(result[0].block_type, "question");
        assert_eq!(result[1].block_type, "response");
    }
    
    #[test]
    fn test_question_with_multiline_content() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question name="recursion-explanation" model="gpt-4">
  Can you explain:
  1. The concept of recursion
  2. How to implement a recursive function
  3. When to use recursion vs iteration
  </meta:question>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse question with multiline content: {:?}", result.err());
        
        let blocks = result.unwrap();
        let block = &blocks[0];
        assert_eq!(block.block_type, "question");
        assert_eq!(block.content.lines().count(), 4);
    }
    
    #[test]
    fn test_simple_response_block() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:response name="simple-response" for="simple-question">
  This is a simple response.
  </meta:response>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse simple response: {:?}", result.err());
        
        let blocks = result.unwrap();
        let block = &blocks[0];
        assert_eq!(block.block_type, "response");
        assert_eq!(block.content, "This is a simple response.");
    }
}
