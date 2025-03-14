#[cfg(test)]
mod executor_results_tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
    
    /// Test executor's handling of results in context building
    #[test]
    #[ignore]
    fn test_executor_includes_results_in_context() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a document with an executable block, results, and a question referencing results
        let input = r#"[code:python name:data-generator]
print([10, 20, 30, 40, 50])
[/code:python]

[results for:data-generator format:plain]
[10, 20, 30, 40, 50]
[/results]

[question name:analyze-data]
Analyze this data: ${data-generator.results}
[/question]"#;
        
        let blocks = parse_document(input).unwrap();
        
        // Mock execution by adding the output to the executor's outputs map
        executor.outputs.insert("data-generator.results".to_string(), "[10, 20, 30, 40, 50]".to_string());
        
        // Find the question block
        let question_block = blocks.iter().find(|b| b.name == Some("analyze-data".to_string())).unwrap();
        
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
        
        // Check that executor correctly applies display modifiers
        assert!(executor.should_display_inline(&inline_block));
        assert!(!executor.should_display_inline(&block_display));
        assert!(!executor.should_display(&none_display));
    }
    
    /// Test executor application of format modifiers
    #[test]
    fn test_executor_applies_format_modifiers() {
        let mut executor = MetaLanguageExecutor::new();
        
        // JSON input with format specified
        let json_input = r#"{"name": "Test", "values": [1, 2, 3]}"#;
        let mut json_block = Block::new("results", None, json_input);
        json_block.add_modifier("for", "json-example");
        json_block.add_modifier("format", "json");
        
        // CSV input with format specified
        let csv_input = "name,age\nJohn,30\nAlice,25";
        let mut csv_block = Block::new("results", None, csv_input);
        csv_block.add_modifier("for", "csv-example");
        csv_block.add_modifier("format", "csv");
        
        // Test auto-detection of format (when no format modifier is specified)
        let auto_json_input = r#"{"auto": "detect"}"#;
        let mut auto_block = Block::new("results", None, auto_json_input);
        auto_block.add_modifier("for", "auto-example");
        
        // Check that executor correctly determines formats
        assert_eq!(executor.determine_format(&json_block), "json");
        assert_eq!(executor.determine_format(&csv_block), "csv");
        assert_eq!(executor.determine_format(&auto_block), "json"); // Auto-detected as JSON
    }
    
    /// Test executor application of trim and max_lines modifiers
    #[test]
    fn test_executor_applies_content_modifiers() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Content with whitespace for trim testing
        let whitespace_content = "\n   Content with leading/trailing whitespace   \n\n";
        let mut trim_block = Block::new("results", None, whitespace_content);
        trim_block.add_modifier("for", "trim-example");
        trim_block.add_modifier("trim", "true");
        
        // Long content for max_lines testing
        let long_content = (0..20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
        let mut max_lines_block = Block::new("results", None, long_content.as_str());
        max_lines_block.add_modifier("for", "max-lines-example");
        max_lines_block.add_modifier("max_lines", "5");
        
        // Apply processing
        let trimmed = executor.apply_trim(&trim_block, whitespace_content);
        let truncated = executor.apply_max_lines(&max_lines_block, &long_content);
        
        // Verify trimming
        assert_eq!(trimmed, "Content with leading/trailing whitespace");
        
        // Verify line truncation
        let truncated_lines = truncated.lines().count();
        assert_eq!(truncated_lines, 5);
        assert!(truncated.contains("Line 0"));
        assert!(truncated.contains("Line 4"));
        assert!(!truncated.contains("Line 5"));
    }
    
    /// Test executor's processing chain for results blocks
    #[test]
    fn test_executor_results_processing_chain() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a block with multiple modifiers
        let raw_content = r#"
{
  "name": "Test Project",
  "values": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
  "description": "This is a test with multiple lines\nthat should be processed together\nwith all modifiers applied correctly."
}
"#;
        
        let mut complex_block = Block::new("results", None, raw_content);
        complex_block.add_modifier("for", "complex-example");
        complex_block.add_modifier("format", "json");
        complex_block.add_modifier("display", "block");
        complex_block.add_modifier("trim", "true");
        complex_block.add_modifier("max_lines", "5");
        
        // Process the content through all the modifier handlers
        let processed = executor.process_results_content(&complex_block, raw_content);
        
        // Verify that all modifiers were applied
        // 1. Should be trimmed
        assert!(!processed.starts_with("\n"));
        
        // 2. Should be truncated to 5 lines
        let lines = processed.lines().count();
        assert!(lines <= 5);
        
        // 3. Should still be valid JSON format
        assert!(processed.contains(r#""name": "Test Project""#));
    }
    
    /// Test integration of results blocks in workflow execution
    #[test]
    #[ignore]
    fn test_results_integration_in_workflow() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a sequence of blocks that forms a basic workflow
        let input = r#"[code:python name:step1]
data = [1, 2, 3, 4, 5]
print(f"Initial data: {data}")
[/code:python]

[results for:step1 format:plain]
Initial data: [1, 2, 3, 4, 5]
[/results]

[code:python name:step2 depends:step1]
data = eval('''${step1.results}'''.split(": ")[1])
processed = [x * 2 for x in data]
print(f"Processed data: {processed}")
[/code:python]

[results for:step2 format:plain]
Processed data: [2, 4, 6, 8, 10]
[/results]

[code:python name:step3 depends:step2]
data = eval('''${step2.results}'''.split(": ")[1])
total = sum(data)
print(f"Total: {total}")
[/code:python]

[results for:step3 format:plain]
Total: 30
[/results]

[question depends:step3]
Analyze the results of this data processing workflow:
Initial data: ${step1.results}
Processed data: ${step2.results}
Final result: ${step3.results}
[/question]"#;
        
        let blocks = parse_document(input).unwrap();
        
        // Mock execution by adding outputs
        executor.outputs.insert("step1.results".to_string(), "Initial data: [1, 2, 3, 4, 5]".to_string());
        executor.outputs.insert("step2.results".to_string(), "Processed data: [2, 4, 6, 8, 10]".to_string());
        executor.outputs.insert("step3.results".to_string(), "Total: 30".to_string());
        
        // Find the question block
        let question_block = blocks.iter().find(|b| b.block_type == "question").unwrap();
        
        // Process variable references
        let processed = executor.process_variable_references(&question_block.content);
        
        // Verify all references are resolved
        assert!(processed.contains("Initial data: Initial data: [1, 2, 3, 4, 5]"));
        assert!(processed.contains("Processed data: Processed data: [2, 4, 6, 8, 10]"));
        assert!(processed.contains("Final result: Total: 30"));
    }
    
    /// Test executor's handling of error_results
    #[test]
    #[ignore]
    fn test_executor_handles_error_results() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a document with a failing executable block and referencing the error
        let input = r#"[code:python name:will-fail]
print(undefined_variable)  # This will cause an error
[/code:python]

[error_results for:will-fail]
Traceback (most recent call last):
  File "<string>", line 1, in <module>
NameError: name 'undefined_variable' is not defined
[/error_results]

[question]
What went wrong with the code? Here's the error: ${will-fail.error_results}
[/question]"#;
        
        let blocks = parse_document(input).unwrap();
        
        // Mock execution error
        let error_msg = "Traceback (most recent call last):\n  File \"<string>\", line 1, in <module>\nNameError: name 'undefined_variable' is not defined";
        executor.outputs.insert("will-fail.error_results".to_string(), error_msg.to_string());
        
        // Find the question block
        let question_block = blocks.iter().find(|b| b.block_type == "question").unwrap();
        
        // Process variable references
        let processed = executor.process_variable_references(&question_block.content);
        
        // Verify error results are included
        assert!(processed.contains("What went wrong with the code? Here's the error:"));
        assert!(processed.contains("NameError: name 'undefined_variable' is not defined"));
    }
    
}
