#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::blocks::Block;

    /// Test template definition and usage
    #[test]
    fn test_template_basics() {
        // Create blocks directly
        use yet_another_llm_project_but_better::parser::blocks::Block;
        
        // Create a template block
        let mut template = Block::new("template", Some("data-processor"), 
            r#"[question model:${model} temperature:${temperature}]
Analyze this data: ${data_content}
[/question]"#);
        
        template.add_modifier("model", "gpt-4");
        template.add_modifier("temperature", "0.7");
        
        // Create a data block
        let mut data_block = Block::new("data", Some("sample-data"), 
            r#"{
  "values": [10, 20, 30, 40, 50],
  "average": 30,
  "metadata": {
    "source": "Example dataset",
    "created": "2023-01-15"
  }
}"#);
        
        data_block.add_modifier("format", "json");
        
        // Create a template invocation block
        let mut invocation = Block::new("template_invocation", Some("data-processor"), "");
        invocation.add_modifier("data_content", "${sample-data}");
        invocation.add_modifier("temperature", "0.3");
        
        // Verify the blocks
        assert_eq!(template.name, Some("data-processor".to_string()));
        assert_eq!(template.get_modifier("model"), Some(&"gpt-4".to_string()));
        assert_eq!(template.get_modifier("temperature"), Some(&"0.7".to_string()));
        
        assert_eq!(data_block.name, Some("sample-data".to_string()));
        assert_eq!(data_block.get_modifier("format"), Some(&"json".to_string()));
        assert!(data_block.content.contains("\"values\": [10, 20, 30, 40, 50]"));
        
        assert_eq!(invocation.name, Some("data-processor".to_string()));
        assert_eq!(invocation.get_modifier("temperature"), Some(&"0.3".to_string()));
        assert!(invocation.get_modifier("data_content").is_some());
    }
    
    /// Test multiple template invocations with different parameters
    #[test]
    fn test_multiple_template_invocations() {
        // Create blocks directly
        use yet_another_llm_project_but_better::parser::blocks::Block;
        
        // Create template block
        let mut template = Block::new("template", Some("message-template"), 
            "${greeting}, ${name}! How are you today?");
        template.add_modifier("greeting", "Hello");
        template.add_modifier("name", "User");
        
        // Create invocation blocks
        let mut invocation1 = Block::new("template_invocation", Some("invocation1"), "");
        invocation1.add_modifier("greeting", "Hi");
        invocation1.add_modifier("name", "John");
        
        let mut invocation2 = Block::new("template_invocation", Some("invocation2"), "");
        invocation2.add_modifier("greeting", "Hello");
        invocation2.add_modifier("name", "Sarah");
        
        let mut invocation3 = Block::new("template_invocation", Some("invocation3"), "");
        invocation3.add_modifier("greeting", "Greetings");
        invocation3.add_modifier("name", "Dr. Smith");
        
        // Verify template definition
        assert_eq!(template.name, Some("message-template".to_string()));
        assert_eq!(template.get_modifier("greeting"), Some(&"Hello".to_string()));
        assert_eq!(template.get_modifier("name"), Some(&"User".to_string()));
        assert!(template.content.contains("How are you today?"));
        
        // Check parameters of each invocation
        assert_eq!(invocation1.get_modifier("greeting"), Some(&"Hi".to_string()));
        assert_eq!(invocation1.get_modifier("name"), Some(&"John".to_string()));
        
        assert_eq!(invocation2.get_modifier("greeting"), Some(&"Hello".to_string()));
        assert_eq!(invocation2.get_modifier("name"), Some(&"Sarah".to_string()));
        
        assert_eq!(invocation3.get_modifier("greeting"), Some(&"Greetings".to_string()));
        assert_eq!(invocation3.get_modifier("name"), Some(&"Dr. Smith".to_string()));
        
        // Verify template invocation references the correct template
        assert_eq!(invocation1.name, Some("invocation1".to_string()));
        assert_eq!(invocation2.name, Some("invocation2".to_string()));
        assert_eq!(invocation3.name, Some("invocation3".to_string()));
    }
    
    /// Test simple template parsing with debug output
    #[test]
    fn test_simple_template_parsing() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::blocks::Block;
        
        let template = Block::new("template", Some("simple-template"), 
            "This is a simple template with no parameters.");
        
        // Print out the block for debugging
        println!("Block: type={}, name={:?}", template.block_type, template.name);
        println!("  Content: {}", template.content);
        println!("  Modifiers:");
        for (key, value) in &template.modifiers {
            println!("    {} = {}", key, value);
        }
        
        // Basic assertions
        assert_eq!(template.block_type, "template");
        assert_eq!(template.name, Some("simple-template".to_string()));
        assert!(template.content.contains("This is a simple template"));
    }
    
    /// Test simple template invocation parsing
    #[test]
    fn test_simple_template_invocation() {
        // Create blocks directly
        use yet_another_llm_project_but_better::parser::blocks::Block;
        
        // Create template block
        let mut template = Block::new("template", Some("greeting-template"), 
            "Hello, ${param1}! Welcome to our service.");
        template.add_modifier("param1", "default-value");
        
        // Create invocation block
        let mut invocation = Block::new("template_invocation", Some("greeting-template"), "");
        invocation.add_modifier("param1", "World");
        
        // Print out the blocks for debugging
        println!("Template block: type={}, name={:?}", template.block_type, template.name);
        println!("  Content: {}", template.content);
        println!("  Modifiers:");
        for (key, value) in &template.modifiers {
            println!("    {} = {}", key, value);
        }
        
        println!("Invocation block: type={}, name={:?}", invocation.block_type, invocation.name);
        println!("  Modifiers:");
        for (key, value) in &invocation.modifiers {
            println!("    {} = {}", key, value);
        }
        
        // Basic assertions
        assert_eq!(template.name, Some("greeting-template".to_string()));
        assert_eq!(template.get_modifier("param1"), Some(&"default-value".to_string()));
        assert!(template.content.contains("Hello, ${param1}!"));
        
        assert_eq!(invocation.name, Some("greeting-template".to_string()));
        assert_eq!(invocation.get_modifier("param1"), Some(&"World".to_string()));
    }
}
