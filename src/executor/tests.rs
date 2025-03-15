#[cfg(test)]
mod tests {
    use crate::executor::MetaLanguageExecutor;
    use crate::parser::Block;
    use std::time::{Duration, Instant};
    
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
        
        // Print block information before execution
        println!("Executing block 'process-data'");
        println!("Block content: {:?}", executor.blocks.get("process-data").map(|b| &b.content));
        
        // Execute the code block
        let result = executor.execute_block("process-data");
        
        // Print detailed execution result
        println!("Execution result: {:?}", result);
        
        // Verify the result
        assert!(result.is_ok(), "Block execution should succeed");
        if let Ok(output) = result {
            println!("Output: '{}'", output);
            assert!(output.trim() == "15", "Sum of [1,2,3,4,5] should be 15, got '{}' instead", output.trim());
        } else if let Err(err) = &result {
            println!("Error details: {:?}", err);
        }
    }
    
    /// Test test mode via modifier
    #[test]
    fn test_test_mode_via_modifier() {
        // Mock executor outputs directly
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a test block with test_mode:true
        let mut block = Block::new("question", Some("test_question"), "What is the meaning of life?");
        block.add_modifier("test_mode", "true");
        
        // Execute the block
        let result = executor.execute_question(&block, &block.content);
        
        // Verify we get the test response
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a simulated response for testing purposes.");
    }
    
    /// Test custom test response
    #[test]
    fn test_custom_test_response() {
        // Mock executor outputs directly
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a test block with test_mode:true and custom test_response
        let mut block = Block::new("question", Some("test_question"), "What is the meaning of life?");
        block.add_modifier("test_mode", "true");
        block.add_modifier("test_response", "The answer is 42.");
        
        // Execute the block
        let result = executor.execute_question(&block, &block.content);
        
        // Verify we get the custom test response
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "The answer is 42.");
    }
    
    /// Test timeout from modifier
    #[test]
    fn test_timeout_from_modifier() {
        let executor = MetaLanguageExecutor::new();
        
        // Create a block with a timeout modifier
        let mut block = Block::new("code:python", Some("test_timeout"), "print('Hello')");
        block.add_modifier("timeout", "30");
        
        // Get the timeout
        let timeout = executor.get_timeout(&block);
        
        // Verify the timeout is set correctly
        assert_eq!(timeout, Duration::from_secs(30));
    }
    
    /// Test caching
    #[test]
    fn test_caching() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a block with caching enabled
        let mut block = Block::new("code:python", Some("cached_block"), "print('Hello')");
        block.add_modifier("cache_result", "true");
        
        // Add the block to the executor
        executor.blocks.insert("cached_block".to_string(), block);
        
        // Simulate a cached result
        executor.cache.insert(
            "cached_block".to_string(), 
            ("Cached result".to_string(), Instant::now())
        );
        
        // Execute the block
        let result = executor.execute_block("cached_block");
        
        // Verify we get the cached result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cached result");
    }
}
