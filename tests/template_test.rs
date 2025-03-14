#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document};

    /// Test template definition and usage
    #[test]
    fn test_template_basics() {
        let input = r#"[template name:data-processor model:gpt-4 temperature:0.7]
[question model:${model} temperature:${temperature}]
Analyze this data: ${data_content}
[/question]
[/template]

[data name:sample-data format:json]
{
  "values": [10, 20, 30, 40, 50],
  "average": 30,
  "metadata": {
    "source": "Example dataset",
    "created": "2023-01-15"
  }
}
[/data]

[@data-processor data_content:"${sample-data}" temperature:"0.3"]
[/@data-processor]"#;

        let blocks = parse_document(input).unwrap();
        
        // Should have 2 blocks: template and data
        assert_eq!(blocks.len(), 2);
        
        // Verify the template block
        let template = blocks.iter().find(|b| b.block_type == "template").unwrap();
        assert_eq!(template.name, Some("data-processor".to_string()));
        assert_eq!(template.get_modifier("model"), Some(&"gpt-4".to_string()));
        assert_eq!(template.get_modifier("temperature"), Some(&"0.7".to_string()));
        
        // Verify the data block
        let data_block = blocks.iter().find(|b| b.block_type == "data").unwrap();
        assert_eq!(data_block.name, Some("sample-data".to_string()));
        assert_eq!(data_block.get_modifier("format"), Some(&"json".to_string()));
        assert!(data_block.content.contains("\"values\": [10, 20, 30, 40, 50]"));
        
        // Verify the template invocation
        let invocation = blocks.iter().find(|b| b.block_type == "template_invocation").unwrap();
        assert_eq!(invocation.name, Some("data-processor".to_string()));
        assert_eq!(invocation.get_modifier("temperature"), Some(&"0.3".to_string()));
        assert!(invocation.get_modifier("data_content").is_some());
    }
    
    /// Test multiple template invocations with different parameters
    #[test]
    fn test_multiple_template_invocations() {
        let input = r#"[template name:message-template greeting:Hello name:User]
${greeting}, ${name}! How are you today?
[/template]

[@message-template name:invocation1 greeting:"Hi" name:"John"]
[/@message-template]

[@message-template name:invocation2 greeting:"Hello" name:"Sarah"]
[/@message-template]

[@message-template name:invocation3 greeting:"Greetings" name:"Dr. Smith"]
[/@message-template]"#;

        let blocks = parse_document(input).unwrap();
        
        // Should have 4 blocks: template and 3 invocations
        assert_eq!(blocks.len(), 4);
        
        // Verify template definition
        let template = blocks.iter().find(|b| b.block_type == "template").unwrap();
        assert_eq!(template.name, Some("message-template".to_string()));
        assert_eq!(template.get_modifier("greeting"), Some(&"Hello".to_string()));
        assert_eq!(template.get_modifier("name"), Some(&"User".to_string()));
        assert!(template.content.contains("How are you today?"));
        
        // Check each invocation
        let invocations = blocks.iter()
            .filter(|b| b.block_type == "template_invocation")
            .collect::<Vec<_>>();
            
        assert_eq!(invocations.len(), 3);
        
        // Check parameters of each invocation
        assert_eq!(invocations[0].get_modifier("greeting"), Some(&"Hi".to_string()));
        assert_eq!(invocations[0].get_modifier("name"), Some(&"John".to_string()));
        
        assert_eq!(invocations[1].get_modifier("greeting"), Some(&"Hello".to_string()));
        assert_eq!(invocations[1].get_modifier("name"), Some(&"Sarah".to_string()));
        
        assert_eq!(invocations[2].get_modifier("greeting"), Some(&"Greetings".to_string()));
        assert_eq!(invocations[2].get_modifier("name"), Some(&"Dr. Smith".to_string()));
        
        // Verify template invocation references the correct template
        assert_eq!(invocations[0].name, Some("invocation1".to_string()));
        assert_eq!(invocations[1].name, Some("invocation2".to_string()));
        assert_eq!(invocations[2].name, Some("invocation3".to_string()));
    }
    
    /// Test simple template parsing with debug output
    #[test]
    fn test_simple_template_parsing() {
        let input = r#"[template name:simple-template]
This is a simple template with no parameters.
[/template]"#;

        let blocks = parse_document(input).unwrap();
        
        // Print out the parsed blocks for debugging
        println!("Number of blocks parsed: {}", blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            println!("Block {}: type={}, name={:?}", 
                i, 
                block.block_type, 
                block.name
            );
            println!("  Content: {}", block.content);
            println!("  Modifiers:");
            for (key, value) in &block.modifiers {
                println!("    {} = {}", key, value);
            }
        }
        
        // Basic assertions
        assert!(!blocks.is_empty(), "Should parse at least one block");
        
        // Find a template block
        let template = blocks.iter().find(|b| b.block_type == "template");
        assert!(template.is_some(), "Should find a template block");
        
        if let Some(template) = template {
            assert_eq!(template.name, Some("simple-template".to_string()));
            assert!(template.content.contains("This is a simple template"));
        }
    }
    
    /// Test simple template invocation parsing
    #[test]
    fn test_simple_template_invocation() {
        let input = r#"[template name:greeting-template param1:default-value]
Hello, ${param1}! Welcome to our service.
[/template]

[@greeting-template param1:"World"]
[/@greeting-template]"#;

        let blocks = parse_document(input).unwrap();
        
        // Print out the parsed blocks for debugging
        println!("Number of blocks parsed: {}", blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            println!("Block {}: type={}, name={:?}", 
                i, 
                block.block_type, 
                block.name
            );
            println!("  Content: {}", block.content);
            println!("  Modifiers:");
            for (key, value) in &block.modifiers {
                println!("    {} = {}", key, value);
            }
        }
        
        // Basic assertions
        assert_eq!(blocks.len(), 2, "Should parse exactly two blocks");
        
        // Find the template block
        let template = blocks.iter().find(|b| b.block_type == "template").unwrap();
        assert_eq!(template.name, Some("greeting-template".to_string()));
        assert_eq!(template.get_modifier("param1"), Some(&"default-value".to_string()));
        assert!(template.content.contains("Hello, ${param1}!"));
        
        // Find the template invocation block
        let invocation = blocks.iter().find(|b| b.block_type == "template_invocation").unwrap();
        assert_eq!(invocation.name, Some("invoke-greeting-template".to_string()));
        assert_eq!(invocation.get_modifier("template"), Some(&"greeting-template".to_string()));
        assert_eq!(invocation.get_modifier("param1"), Some(&"World".to_string()));
    }
}
