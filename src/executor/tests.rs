#[cfg(test)]
mod tests {
    use crate::executor::MetaLanguageExecutor;
    
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
}
