#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, extract_variable_references};
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;

    /// Test basic variable resolution with the executor
    #[test]
    fn test_executor_variable_resolution() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Set up some variables
        executor.outputs.insert("name".to_string(), "John".to_string());
        executor.outputs.insert("age".to_string(), "30".to_string());
        executor.outputs.insert("language".to_string(), "Python".to_string());
        
        // Process a string with variable references
        let input = "Hello, ${name}! I see you are ${age} years old and like ${language}.";
        let processed = executor.process_variable_references(input);
        
        // Check that variables were properly replaced
        assert_eq!(processed, "Hello, John! I see you are 30 years old and like Python.");
    }
    
    /// Test variable extraction from strings
    #[test]
    fn test_variable_reference_extraction() {
        // Simple string with variable references
        let input = "This ${text} contains ${multiple} variable ${references}.";
        let references = extract_variable_references(input);
        
        // Should extract all variables
        assert_eq!(references.len(), 3);
        assert!(references.contains(&"text".to_string()));
        assert!(references.contains(&"multiple".to_string()));
        assert!(references.contains(&"references".to_string()));
    }
    
    /// Test basic parsing of blocks with variable references
    #[test]
    fn test_parse_blocks_with_variables() {
        let input = r#"[variable name:greeting]
Hello, world!
[/variable]

[variable name:signature]
Best regards,
The Team
[/variable]

[data name:user-info format:json]
{
  "name": "Jane Doe",
  "email": "jane@example.com"
}
[/data]

[code:python name:generate-email]
user_data = '''${user-info}'''
greeting = '''${greeting}'''
signature = '''${signature}'''

print(f"Email preview:\n\n{greeting}\n\nDear {user_data['name']},\n\nWelcome to our platform!\n\n{signature}")
[/code:python]"#;

        let blocks = parse_document(input).unwrap();
        
        // Check that all blocks are parsed
        assert_eq!(blocks.len(), 4);
        
        // Get the code block and check for variable references
        let code_block = blocks.iter().find(|b| b.name == Some("generate-email".to_string())).unwrap();
        
        // Extract variable references
        let references = extract_variable_references(&code_block.content);
        assert_eq!(references.len(), 3);
        assert!(references.contains(&"user-info".to_string()));
        assert!(references.contains(&"greeting".to_string()));
        assert!(references.contains(&"signature".to_string()));
    }
}
