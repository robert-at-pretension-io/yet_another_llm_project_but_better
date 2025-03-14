#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, ParserError};
    use yet_another_llm_project_but_better::executor::{MetaLanguageExecutor, ExecutorError};
    use yet_another_llm_project_but_better::parser::Block;
    
    #[test]
     // Temporarily ignore this test until we fix block parsing
    fn test_parser_invalid_block_structure() {
        // Missing closing tag
        let input = r#"[data name:invalid-block]
This block is missing its closing tag
"#;
        
        let result = parse_document(input);
        assert!(result.is_err(), "Expected error for invalid block structure");
        
        match result {
            Err(ParserError::InvalidBlockStructure(_)) => assert!(true),
            _ => panic!("Expected InvalidBlockStructure error"),
        }
    }
    
    #[test]
    fn test_parser_duplicate_block_names() {
        // Two blocks with the same name
        let input = r#"[data name:duplicate]
First block
[/data]

[code:python name:duplicate]
print("Second block with duplicate name")
[/code:python]"#;
        
        let result = parse_document(input);
        assert!(result.is_err(), "Expected error for duplicate block names");
        
        match result {
            Err(ParserError::DuplicateBlockName(_)) => assert!(true),
            _ => panic!("Expected DuplicateBlockName error"),
        }
    }
    
    #[test]
    fn test_executor_block_not_found() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Try to execute a non-existent block
        let result = executor.execute_block("non-existent-block");
        assert!(result.is_err(), "Expected error for non-existent block");
        
        match result {
            Err(ExecutorError::BlockNotFound(_)) => assert!(true),
            _ => panic!("Expected BlockNotFound error"),
        }
    }
    
    #[test]
    fn test_executor_circular_dependency() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create blocks with circular dependencies
        let mut block_a = Block::new("code:python", Some("block-a"), "print('${block-b}')");
        block_a.add_modifier("depends", "block-b");
        
        let mut block_b = Block::new("code:python", Some("block-b"), "print('${block-a}')");
        block_b.add_modifier("depends", "block-a");
        
        // Add blocks to executor
        executor.blocks.insert("block-a".to_string(), block_a);
        executor.blocks.insert("block-b".to_string(), block_b);
        
        // Try to execute block with circular dependency
        let result = executor.execute_block("block-a");
        assert!(result.is_err(), "Expected error for circular dependency");
        
        match result {
            Err(ExecutorError::CircularDependency(_)) => assert!(true),
            _ => panic!("Expected CircularDependency error"),
        }
    }
}
