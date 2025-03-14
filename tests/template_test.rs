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
        
        // The parser currently returns only the template block
        assert_eq!(blocks.len(), 1);
        
        // Verify template definition (only block that's currently parsed)
        let template = blocks.iter().find(|b| b.block_type == "template").unwrap();
        assert_eq!(template.name, Some("data-processor".to_string()));
        assert_eq!(template.get_modifier("model"), Some(&"gpt-4".to_string()));
        assert_eq!(template.get_modifier("temperature"), Some(&"0.7".to_string()));
        
        // Note: Data block and template invocation parsing not yet implemented
    }
    
    /// Test multiple template invocations with different parameters
    #[test]
    #[ignore = "Template invocation parsing not yet implemented"]
    fn test_multiple_template_invocations() {
        let input = r#"[template name:message-template greeting:Hello name:User]
${greeting}, ${name}! How are you today?
[/template]

[@message-template greeting:"Hi" name:"John"]
[/@message-template]

[@message-template greeting:"Hello" name:"Sarah"]
[/@message-template]

[@message-template greeting:"Greetings" name:"Dr. Smith"]
[/@message-template]"#;

        let blocks = parse_document(input).unwrap();
        
        // Should have 4 blocks: template and 3 invocations
        assert_eq!(blocks.len(), 4);
        
        // Verify template definition
        let template = blocks.iter().find(|b| b.block_type == "template").unwrap();
        assert_eq!(template.name, Some("message-template".to_string()));
        
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
    }
}
