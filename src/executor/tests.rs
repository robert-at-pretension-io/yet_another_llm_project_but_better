#[cfg(test)]
mod tests {
    use super::*;

    /// Test variable references to results blocks
    #[test]
    fn test_variable_reference_to_results() {
        // Mock executor outputs directly
        let mut executor = MetaLanguageExecutor::new();
        executor.outputs.insert("generate-data.results".to_string(), "[1, 2, 3, 4, 5]".to_string());
        
        // Test variable resolution with the mock data
        let content = "import json\ndata = ${generate-data.results}\nprint(f\"Sum: {sum(data)}\")";
        let processed = executor.process_variable_references(content);
        
        // Print the processed content for debugging
        println!("Processed content: {}", processed);
        
        // Check that the variable was replaced correctly
        assert_eq!(processed.contains("${generate-data.results}"), false);
        assert_eq!(processed.contains("[1, 2, 3, 4, 5]"), true);
    }
}
