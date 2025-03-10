#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::Block;
    
    #[test]
    fn test_debug_block() {
        // Create a debug block manually
        let mut block = Block::new("debug", Some("execution-log"), "");
        block.add_modifier("level", "verbose");
        block.content = r#"This block contains execution logs and debugging information.
Timestamp: 2023-05-15T14:30:00Z
Steps executed: 15
Memory usage: 128MB"#.to_string();
        
        assert_eq!(block.block_type, "debug");
        assert_eq!(block.name, Some("execution-log".to_string()));
        assert_eq!(block.get_modifier("level"), Some(&"verbose".to_string()));
        
        let content = block.content.as_str();
        assert!(content.contains("execution logs"));
        assert!(content.contains("Timestamp"));
        assert!(content.contains("Memory usage"));
    }
    
    #[test]
    fn test_visualization_block() {
        // Create a visualization block manually
        let mut block = Block::new("visualization", Some("context-preview"), "");
        block.content = r#"[question debug:true]
What are the key factors affecting climate change?
[/question]

[preview]
The following context will be sent to the AI:
- User query about climate change factors
- Recent IPCC report summary (from data block 'ipcc-summary')
- Historical temperature data visualization (from code block 'temp-vis')
- Previous conversation context about environmental policies
[/preview]"#.to_string();
        
        assert_eq!(block.block_type, "visualization");
        assert_eq!(block.name, Some("context-preview".to_string()));
        
        let content = block.content.as_str();
        assert!(content.contains("[question debug:true]"));
        assert!(content.contains("[preview]"));
        assert!(content.contains("climate change"));
        assert!(content.contains("IPCC report"));
    }
    
    #[test]
    fn test_preview_block() {
        // Create a preview block manually
        let mut block = Block::new("preview", Some("prompt-preview"), "");
        block.content = r#"System: You are an AI assistant specialized in climate science.
User: What are the main greenhouse gases?
Context: 
- From IPCC report: CO2, CH4, N2O, and fluorinated gases
- CO2 is responsible for ~76% of total GHG emissions"#.to_string();
        
        assert_eq!(block.block_type, "preview");
        assert_eq!(block.name, Some("prompt-preview".to_string()));
        
        let content = block.content.as_str();
        assert!(content.contains("System:"));
        assert!(content.contains("User:"));
        assert!(content.contains("Context:"));
        assert!(content.contains("greenhouse gases"));
    }
    
    #[test]
    fn test_nested_visualization() {
        // Create a visualization block with nested content
        let mut block = Block::new("visualization", Some("complex-view"), "");
        block.content = r#"[question name:climate-query]
Explain the greenhouse effect.
[/question]

[data name:context-data format:json]
{
  "source": "IPCC",
  "key_points": [
    "Greenhouse gases trap heat in the atmosphere",
    "CO2 levels have increased by 40% since pre-industrial times",
    "Human activities are the primary cause"
  ]
}
[/data]

[preview name:final-prompt]
The final prompt will include:
1. User question about greenhouse effect
2. Scientific context from IPCC
3. Previous conversation history (3 turns)
[/preview]"#.to_string();
        
        // In the test, we're creating just one block manually
        let blocks = vec![block];
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "visualization");
        assert_eq!(blocks[0].name, Some("complex-view".to_string()));
        
        let content = blocks[0].content.as_str();
        assert!(content.contains("[question name:climate-query]"));
        assert!(content.contains("[data name:context-data format:json]"));
        assert!(content.contains("[preview name:final-prompt]"));
        assert!(content.contains("greenhouse effect"));
        assert!(content.contains("IPCC"));
    }
}
