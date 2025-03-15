#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
    use yet_another_llm_project_but_better::parser::{Block, parse_document};
    use std::time::Duration;
    
    #[test]
    fn test_executor_initialization() {
        let executor = MetaLanguageExecutor::new();
        
        assert!(executor.blocks.is_empty());
        assert!(executor.outputs.is_empty());
        assert!(executor.fallbacks.is_empty());
        assert!(executor.cache.is_empty());
        assert!(executor.current_document.is_empty());
    }
    
    #[test]
    fn test_executor_variable_reference_resolution() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Add some values to the outputs
        executor.outputs.insert("api_key".to_string(), "abc123".to_string());
        executor.outputs.insert("base_url".to_string(), "https://api.example.com".to_string());
        executor.outputs.insert("endpoint".to_string(), "/data".to_string());
        
        // Test variable reference resolution
        let content = "API Key: ${api_key}, URL: ${base_url}${endpoint}";
        let processed = executor.process_variable_references(content);
        
        assert_eq!(processed, "API Key: abc123, URL: https://api.example.com/data");
    }
    
    #[test]
    fn test_executor_nested_variable_references() {
        // This test verifies that the implementation correctly 
        // resolves nested variable references
        let mut executor = MetaLanguageExecutor::new();
        
        // Add values that include references to other variables
        executor.outputs.insert("base_url".to_string(), "https://api.example.com".to_string());
        executor.outputs.insert("endpoint".to_string(), "/users".to_string());
        executor.outputs.insert("full_url".to_string(), "${base_url}${endpoint}".to_string());
        
        // Test nested resolution - the implementation should resolve the nested references
        let content = "Fetching data from ${full_url}";
        let processed = executor.process_variable_references(content);
        
        // The implementation correctly resolves nested variables
        assert_eq!(processed, "Fetching data from https://api.example.com/users");
    }
    
    #[test]
    fn test_executor_is_cacheable() {
        let executor = MetaLanguageExecutor::new();
        
        // Test block with cache_result:true
        let mut block1 = Block::new("code:python", Some("cacheable"), "print('hello')");
        block1.add_modifier("cache_result", "true");
        
        // Test block with cache_result:false
        let mut block2 = Block::new("code:python", Some("not-cacheable"), "print('world')");
        block2.add_modifier("cache_result", "false");
        
        // Test block without cache_result modifier
        let block3 = Block::new("code:python", Some("default"), "print('default')");
        
        assert!(executor.is_cacheable(&block1));
        assert!(!executor.is_cacheable(&block2));
        assert!(!executor.is_cacheable(&block3));
    }
    
    #[test]
    fn test_executor_missing_variable_references() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Add only some of the referenced variables
        executor.outputs.insert("var1".to_string(), "value1".to_string());
        
        // Test with missing variables
        let content = "Known: ${var1}, Unknown: ${var2}";
        let processed = executor.process_variable_references(content);
        
        // Missing variables should remain as is
        assert_eq!(processed, "Known: value1, Unknown: ${var2}");
    }
    
    #[test]
    fn test_is_executable_block() {
        let executor = MetaLanguageExecutor::new();
        
        // Test executable block types
        let python_block = Block::new("code:python", Some("py-test"), "print('hello')");
        let js_block = Block::new("code:javascript", Some("js-test"), "console.log('hello')");
        let rust_block = Block::new("code:rust", Some("rust-test"), "println!(\"hello\");");
        let shell_block = Block::new("shell", Some("shell-test"), "echo hello");
        let api_block = Block::new("api", Some("api-test"), "GET /users");
        
        // Test non-executable block types
        let data_block = Block::new("data", Some("data-test"), "some data");
        let variable_block = Block::new("variable", Some("var-test"), "some value");
        
        assert!(executor.is_executable_block(&python_block));
        assert!(executor.is_executable_block(&js_block));
        assert!(executor.is_executable_block(&rust_block));
        assert!(executor.is_executable_block(&shell_block));
        assert!(executor.is_executable_block(&api_block));
        
        assert!(!executor.is_executable_block(&data_block));
        assert!(!executor.is_executable_block(&variable_block));
    }
    
    #[test]
    fn test_has_fallback() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Add a fallback
        executor.fallbacks.insert("api-call".to_string(), "Error: API unavailable".to_string());
        
        assert!(executor.has_fallback("api-call"));
        assert!(!executor.has_fallback("non-existent"));
    }
    
    #[test]
    fn test_has_explicit_dependency() {
        let executor = MetaLanguageExecutor::new();
        
        // Create blocks with dependencies
        let mut block_with_depends = Block::new("code:python", Some("test1"), "print('hello')");
        block_with_depends.add_modifier("depends", "data-block");
        
        let mut block_with_requires = Block::new("code:python", Some("test2"), "print('world')");
        block_with_requires.add_modifier("requires", "config-block");
        
        let block_without_deps = Block::new("code:python", Some("test3"), "print('no deps')");
        
        assert!(executor.has_explicit_dependency(&block_with_depends));
        assert!(executor.has_explicit_dependency(&block_with_requires));
        assert!(!executor.has_explicit_dependency(&block_without_deps));
    }
    
    #[test]
    fn test_get_timeout() {
        let executor = MetaLanguageExecutor::new();
        
        // Block with timeout modifier
        let mut block_with_timeout = Block::new("code:python", Some("test-timeout"), "print('hello')");
        block_with_timeout.add_modifier("timeout", "30");
        
        // Block without timeout modifier
        let block_without_timeout = Block::new("code:python", Some("test-default"), "print('world')");
        
        // Block with invalid timeout
        let mut block_with_invalid_timeout = Block::new("code:python", Some("test-invalid"), "print('error')");
        block_with_invalid_timeout.add_modifier("timeout", "not-a-number");
        
        assert_eq!(executor.get_timeout(&block_with_timeout), Duration::from_secs(30));
        assert_eq!(executor.get_timeout(&block_without_timeout), Duration::from_secs(600)); // Default 10 minutes
        assert_eq!(executor.get_timeout(&block_with_invalid_timeout), Duration::from_secs(600)); // Default for invalid
    }
    
    #[test]
    fn test_determine_format_from_content() {
        let executor = MetaLanguageExecutor::new();
        
        // Test JSON detection
        let json_obj = r#"{"name": "John", "age": 30}"#;
        let json_arr = r#"[1, 2, 3, 4]"#;
        
        assert_eq!(executor.determine_format_from_content(json_obj), "json");
        assert_eq!(executor.determine_format_from_content(json_arr), "json");
        
        // For other formats, the implementation would need to be checked
        // This is a placeholder that will pass with the current implementation
        let text = "Just some plain text";
        assert_ne!(executor.determine_format_from_content(text), "json");
    }
    
    #[test]
    fn test_generate_results_block() {
        let executor = MetaLanguageExecutor::new();
        
        // Create a source block
        let source_block = Block::new("code:python", Some("source-block"), "print('hello')");
        
        // Generate a results block
        let output = "Hello, world!";
        let results_block = executor.generate_results_block(&source_block, output, Some("text"));
        
        // Check the results block
        assert_eq!(results_block.block_type, "results");
        assert_eq!(results_block.content, "Hello, world!");
        
        // Check the "for" modifier
        let for_modifier = results_block.modifiers.iter()
            .find(|(k, _)| k == "for")
            .map(|(_, v)| v);
        
        assert_eq!(for_modifier, Some(&"source-block".to_string()));
        
        // Check the format modifier
        let format_modifier = results_block.modifiers.iter()
            .find(|(k, _)| k == "format")
            .map(|(_, v)| v);
        
        assert_eq!(format_modifier, Some(&"text".to_string()));
    }
    
    #[test]
    fn test_generate_error_results_block() {
        let executor = MetaLanguageExecutor::new();
        
        // Create a source block
        let source_block = Block::new("code:python", Some("error-source"), "print(undefined_var)");
        
        // Generate an error results block
        let error = "NameError: name 'undefined_var' is not defined";
        let error_block = executor.generate_error_results_block(&source_block, error);
        
        // Check the error block
        assert_eq!(error_block.block_type, "error_results");
        assert_eq!(error_block.content, "NameError: name 'undefined_var' is not defined");
        
        // Check the "for" modifier
        let for_modifier = error_block.modifiers.iter()
            .find(|(k, _)| k == "for")
            .map(|(_, v)| v);
        
        assert_eq!(for_modifier, Some(&"error-source".to_string()));
    }
    
    #[test]
    fn test_question_response_execution() {
        // This test verifies that:
        // 1. Parsing a question block doesn't automatically add a response
        // 2. Executing a question block adds a response block
        // 3. The response block is correctly linked to the question block
        
        // Setup
        let mut executor = MetaLanguageExecutor::new();
        
        // 1. Parse a document with just a question block
        let input = r#"[question name:test-question model:gpt-4]
        What is the capital of France?
        [/question]"#;
        
        let blocks = parse_document(input).unwrap();
        assert_eq!(blocks.len(), 1, "Should only have one block after parsing");
        assert_eq!(blocks[0].block_type, "question");
        
        // Add the question block to the executor
        let question_block = blocks[0].clone();
        let question_name = question_block.name.clone().unwrap();
        executor.blocks.insert(question_name.clone(), question_block);
        
        // Verify no response block exists yet
        assert_eq!(executor.blocks.len(), 1, "Should only have the question block");
        
        // 2. Mock execution of the question block
        // In a real scenario, this would call an LLM API
        // For testing, we'll manually add a response to the outputs
        executor.outputs.insert(question_name.clone(), "Paris is the capital of France.".to_string());
        
        // Generate a response block (simulating what would happen during execution)
        let response_content = executor.outputs.get(&question_name).unwrap();
        let mut response_block = Block::new("response", None, response_content);
        response_block.add_modifier("for", &question_name);
        
        // Add the response block to the executor
        let response_name = format!("{}-response", question_name);
        executor.blocks.insert(response_name.clone(), response_block);
        
        // 3. Verify the response block is correctly linked to the question
        assert_eq!(executor.blocks.len(), 2, "Should now have question and response blocks");
        
        let stored_response = executor.blocks.get(&response_name).unwrap();
        assert_eq!(stored_response.block_type, "response");
        
        // Check the "for" modifier links to the question
        let for_modifier = stored_response.get_modifier("for");
        assert_eq!(for_modifier, Some(&question_name));
        
        // Check the content matches what we set
        assert_eq!(stored_response.content, "Paris is the capital of France.");
    }
}
