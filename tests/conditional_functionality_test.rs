#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::executor::BlockRunner;
    use yet_another_llm_project_but_better::parser::Block;
    use yet_another_llm_project_but_better::executor::runners::conditional::ConditionalRunner;

    #[test]
    fn test_conditional_execute() {
        // Skip all the execution and just test our runner implementations directly
        let mut state = yet_another_llm_project_but_better::executor::ExecutorState::new();
        
        // Create a test environment
        state.outputs.insert("is-admin".to_string(), "true".to_string());
        
        // Create a conditional block that depends on is-admin
        let mut conditional_block = Block::new("conditional", Some("admin-conditional"), "This content is conditionally displayed");
        conditional_block.add_modifier("if", "is-admin");
        
        // Create the runner
        let runner = ConditionalRunner;
        
        // Test that the condition works properly
        let result = runner.execute("admin-conditional", &conditional_block, &mut state);
        assert!(result.is_ok(), "Failed to execute conditional block: {:?}", result.err());
        
        // Verify the conditional content is returned when condition is true
        let output = result.unwrap();
        assert_eq!(output, "This content is conditionally displayed", 
                   "Conditional runner did not return the expected content");
                   
        // Now test with a false condition
        state.outputs.insert("is-admin".to_string(), "false".to_string());
        let result2 = runner.execute("admin-conditional", &conditional_block, &mut state);
        assert!(result2.is_ok(), "Failed to execute conditional block with false condition: {:?}", result2.err());
        
        // Verify empty string is returned when condition is false
        let output2 = result2.unwrap();
        assert_eq!(output2, "", "Conditional runner did not return empty string for false condition");
    }
}