#[cfg(test)]
mod executor_results_tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
    
    /// Test executor's handling of results in context building
    
    #[test]
    fn test_executor_includes_results_in_context() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create blocks directly
        let code_block = Block::new("code:python", Some("data-generator"), "print([10, 20, 30, 40, 50])");
        
        let mut results_block = Block::new("results", None, "[10, 20, 30, 40, 50]");
        results_block.add_modifier("for", "data-generator");
        results_block.add_modifier("format", "plain");
        
        let question_block = Block::new("question", Some("analyze-data"), "Analyze this data: ${data-generator.results}");
        
        // Add blocks to executor
        executor.blocks.insert("data-generator".to_string(), code_block);
        executor.blocks.insert("data-generator.results".to_string(), results_block);
        executor.blocks.insert("analyze-data".to_string(), question_block.clone());
        
        // Mock execution by adding the output to the executor's outputs map
        executor.outputs.insert("data-generator.results".to_string(), "[10, 20, 30, 40, 50]".to_string());
        
        // Process variable references in the question
        let processed = executor.process_variable_references(&question_block.content);
        
        // Verify that the results reference is replaced
        assert_eq!(processed, "Analyze this data: [10, 20, 30, 40, 50]");
    }
    
    /// Test executor application of display modifiers
    
    #[test]
    fn test_executor_applies_display_modifiers() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create results blocks with different display modifiers
        let mut inline_block = Block::new("results", None, "This is an inline result.");
        inline_block.add_modifier("for", "inline-example");
        inline_block.add_modifier("display", "inline");
        
        let mut block_display = Block::new("results", None, "This is a block result.");
        block_display.add_modifier("for", "block-example");
        block_display.add_modifier("display", "block");
        
        let mut none_display = Block::new("results", None, "This result is not displayed.");
        none_display.add_modifier("for", "none-example");
        none_display.add_modifier("display", "none");
        
        // Add blocks to executor
        executor.blocks.insert("inline-example.results".to_string(), inline_block);
        executor.blocks.insert("block-example.results".to_string(), block_display);
        executor.blocks.insert("none-example.results".to_string(), none_display);
        
        // Mock execution by adding outputs to the executor's outputs map
        executor.outputs.insert("inline-example.results".to_string(), "This is an inline result.".to_string());
        executor.outputs.insert("block-example.results".to_string(), "This is a block result.".to_string());
        executor.outputs.insert("none-example.results".to_string(), "This result is not displayed.".to_string());
        
        // Test references with different display modifiers
        let inline_ref = "${inline-example.results}";
        let block_ref = "${block-example.results}";
        let none_ref = "${none-example.results}";
        
        let processed_inline = executor.process_variable_references(inline_ref);
        let processed_block = executor.process_variable_references(block_ref);
        let processed_none = executor.process_variable_references(none_ref);
        
        // Verify that display modifiers are applied as expected
        assert_eq!(processed_inline, "This is an inline result."); 
        assert_eq!(processed_block, "\n```\nThis is a block result.\n```\n");
        assert_eq!(processed_none, "");
    }
    
    /// Test executor application of format modifiers
    
    #[test]
    fn test_executor_applies_format_modifiers() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create results blocks with different format modifiers
        let mut markdown_block = Block::new("results", None, "# Heading\n\n- Item 1\n- Item 2");
        markdown_block.add_modifier("for", "markdown-example");
        markdown_block.add_modifier("format", "markdown");
        
        let mut code_block = Block::new("results", None, "def example():\n    return True");
        code_block.add_modifier("for", "code-example");
        code_block.add_modifier("format", "code");
        code_block.add_modifier("language", "python");
        
        let mut plain_block = Block::new("results", None, "Plain text result.");
        plain_block.add_modifier("for", "plain-example");
        plain_block.add_modifier("format", "plain");
        
        // Add blocks to executor
        executor.blocks.insert("markdown-example.results".to_string(), markdown_block);
        executor.blocks.insert("code-example.results".to_string(), code_block);
        executor.blocks.insert("plain-example.results".to_string(), plain_block);
        
        // Mock execution by adding outputs to the executor's outputs map
        executor.outputs.insert("markdown-example.results".to_string(), "# Heading\n\n- Item 1\n- Item 2".to_string());
        executor.outputs.insert("code-example.results".to_string(), "def example():\n    return True".to_string());
        executor.outputs.insert("plain-example.results".to_string(), "Plain text result.".to_string());
        
        // Test references with different format modifiers
        let markdown_ref = "${markdown-example.results}";
        let code_ref = "${code-example.results}";
        let plain_ref = "${plain-example.results}";
        
        let processed_markdown = executor.process_variable_references(markdown_ref);
        let processed_code = executor.process_variable_references(code_ref);
        let processed_plain = executor.process_variable_references(plain_ref);
        
        // Verify that format modifiers are applied as expected
        assert_eq!(processed_markdown, "# Heading\n\n- Item 1\n- Item 2");
        assert_eq!(processed_code, "```python\ndef example():\n    return True\n```");
        assert_eq!(processed_plain, "Plain text result.");
    }
    
    /// Test executor application of content modifiers
    
    #[test]
    fn test_executor_applies_content_modifiers() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create results blocks with different content modifiers
        let mut preview_block = Block::new("results", None, "This is a long result that should be truncated for preview.");
        preview_block.add_modifier("for", "preview-example");
        preview_block.add_modifier("preview", "true");
        preview_block.add_modifier("preview_length", "20");
        
        let mut full_block = Block::new("results", None, "This is a full result without truncation.");
        full_block.add_modifier("for", "full-example");
        
        // Add blocks to executor
        executor.blocks.insert("preview-example.results".to_string(), preview_block);
        executor.blocks.insert("full-example.results".to_string(), full_block);
        
        // Mock execution by adding outputs to the executor's outputs map
        executor.outputs.insert("preview-example.results".to_string(), 
                               "This is a long result that should be truncated for preview.".to_string());
        executor.outputs.insert("full-example.results".to_string(), 
                               "This is a full result without truncation.".to_string());
        
        // Test references with different content modifiers
        let preview_ref = "${preview-example.results}";
        let full_ref = "${full-example.results}";
        
        let processed_preview = executor.process_variable_references(preview_ref);
        let processed_full = executor.process_variable_references(full_ref);
        
        // Verify that content modifiers are applied as expected
        assert_eq!(processed_preview, "This is a long resul...");
        assert_eq!(processed_full, "This is a full result without truncation.");
    }
    
    /// Test processing chain of results with multiple modifiers
    
    #[test]
    fn test_executor_results_processing_chain() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a result block with multiple modifiers
        let mut complex_block = Block::new("results", None, "def example():\n    # This is an example function\n    return True");
        complex_block.add_modifier("for", "complex-example");
        complex_block.add_modifier("format", "code");
        complex_block.add_modifier("language", "python");
        complex_block.add_modifier("display", "block");
        complex_block.add_modifier("highlight", "true");
        
        // Add block to executor
        executor.blocks.insert("complex-example.results".to_string(), complex_block);
        
        // Mock execution by adding outputs to the executor's outputs map
        executor.outputs.insert("complex-example.results".to_string(), 
                               "def example():\n    # This is an example function\n    return True".to_string());
        
        // Test reference with multiple modifiers
        let complex_ref = "${complex-example.results}";
        
        let processed = executor.process_variable_references(complex_ref);
        
        // Verify that all modifiers are applied in the correct order
        // Specifically, the code should be formatted with python syntax highlighting and displayed as a block
        let expected = "\n```python\ndef example():\n    # This is an example function\n    return True\n```\n";
        assert_eq!(processed, expected);
    }
    
    /// Test integration of results in workflow
    
    #[test]
    fn test_results_integration_in_workflow() {
        let document = r#"
        <meta:data name="initial-data" format="json">
        [1, 2, 3, 4, 5]
        </meta:data>

        <meta:code name="processor" language="python">
        data = ${initial-data}
        print(f"Initial data: {data}")
        processed = [x * 2 for x in data]
        print(f"Processed data: {processed}")
        </meta:code>

        <meta:question name="analysis">
        Initial data: ${processor.results}
        What patterns can you see in the data transformation?
        </meta:question>
        "#;
        
        let mut executor = MetaLanguageExecutor::new();
        let _ = executor.process_document(document);
        
        // Mock execution by adding outputs to the executor's outputs map
        executor.outputs.insert("processor.results".to_string(), 
                               "Initial data: [1, 2, 3, 4, 5]\nProcessed data: [2, 4, 6, 8, 10]".to_string());
        
        let processed = executor.update_document().unwrap();
        
        // Verify that results are integrated in the workflow
        assert!(processed.contains("Initial data: Initial data: [1, 2, 3, 4, 5]"));
        assert!(processed.contains("Processed data: [2, 4, 6, 8, 10]"));
    }
    
    /// Test handling of error results
    
    #[test]
    fn test_executor_handles_error_results() {
        let document = r#"
        <meta:code name="error-code" language="python">
        # This code will produce an error
        print(undefined_variable)
        </meta:code>

        <meta:question name="error-analysis">
        What went wrong with the code execution?
        ${error-code.error_results}
        </meta:question>
        "#;
        
        let mut executor = MetaLanguageExecutor::new();
        let _ = executor.process_document(document);
        
        // Mock execution by adding error outputs to the executor's outputs map
        executor.outputs.insert("error-code.error_results".to_string(), 
                               "NameError: name 'undefined_variable' is not defined".to_string());
        
        let processed = executor.update_document().unwrap();
        
        // Verify that error results are included
        assert!(processed.contains("NameError: name 'undefined_variable' is not defined"));
    }
}
