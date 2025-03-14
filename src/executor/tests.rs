#[cfg(test)]
mod tests {
    use crate::executor::MetaLanguageExecutor;
    use crate::parser::Block;
    
    /// Test variable references to results blocks
    #[test]
    fn test_variable_reference_to_results() {
        // Mock executor outputs directly
        let mut executor = MetaLanguageExecutor::new();
        executor.outputs.insert("generate-data.results".to_string(), "[1, 2, 3, 4, 5]".to_string());
        
        // Test variable resolution with the mock data
        let content = "import json\ndata = ${generate-data.results}\nprint(f\"Sum: {sum(data)}\")";
        
        // Print the output map for debugging
        println!("Output map contents:");
        for (key, value) in &executor.outputs {
            println!("  '{}' => '{}'", key, value);
        }
        
        let processed = executor.process_variable_references(content);
        
        // Print the processed content for debugging
        println!("Processed content: '{}'", processed);
        println!("Original content: '{}'", content);
        
        // Check if the variable reference is gone
        assert!(!processed.contains("${generate-data.results}"), 
                "Variable reference should be replaced");
        
        // Check that the content contains the expected value
        assert!(processed.contains("[1, 2, 3, 4, 5]"), 
                "Processed content should contain the data array");
    }
    
    /// Test block execution with dependencies
    #[test]
    fn test_block_execution_with_dependencies() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a data block
        let mut data_block = Block::new("data", Some("test-data"), "[1, 2, 3, 4, 5]");
        executor.blocks.insert("test-data".to_string(), data_block);
        executor.outputs.insert("test-data".to_string(), "[1, 2, 3, 4, 5]".to_string());
        
        // Create a code block that depends on the data block
        let mut code_block = Block::new("code:python", Some("process-data"), 
            "data = ${test-data}\nresult = sum(data)\nprint(result)");
        code_block.add_modifier("depends", "test-data");
        executor.blocks.insert("process-data".to_string(), code_block);
        
        // Execute the code block
        let result = executor.execute_block("process-data");
        
        // Verify the result
        assert!(result.is_ok(), "Block execution should succeed");
        if let Ok(output) = result {
            assert!(output.trim() == "15", "Sum of [1,2,3,4,5] should be 15");
        }
    }
}
