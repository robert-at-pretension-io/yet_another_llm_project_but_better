#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::parse_document;
    
    #[test]
    #[ignore]
    fn test_execution_control_modifiers() {
        let input = r#"[code:python name:test-exec cache_result:true timeout:30 retry:3 fallback:test-fallback async:true]
print("This is a test with execution modifiers")
[/code:python]"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "code:python");
        assert_eq!(blocks[0].name, Some("test-exec".to_string()));
        
        // Verify specific modifiers
        let cache_result = blocks[0].modifiers.iter().find(|(k, _)| k == "cache_result").map(|(_, v)| v);
        assert_eq!(cache_result, Some(&"true".to_string()));
        
        let timeout = blocks[0].modifiers.iter().find(|(k, _)| k == "timeout").map(|(_, v)| v);
        assert_eq!(timeout, Some(&"30".to_string()));
        
        let retry = blocks[0].modifiers.iter().find(|(k, _)| k == "retry").map(|(_, v)| v);
        assert_eq!(retry, Some(&"3".to_string()));
        
        let fallback = blocks[0].modifiers.iter().find(|(k, _)| k == "fallback").map(|(_, v)| v);
        assert_eq!(fallback, Some(&"test-fallback".to_string()));
        
        let async_mod = blocks[0].modifiers.iter().find(|(k, _)| k == "async").map(|(_, v)| v);
        assert_eq!(async_mod, Some(&"true".to_string()));
    }
    
    #[test]
    #[ignore]
    fn test_context_management_modifiers() {
        let input = r#"[data name:context-data always_include:true priority:8 order:0.5 weight:0.7 summarize:brief]
{
  "context": "This is context data"
}
[/data]"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[0].name, Some("context-data".to_string()));
        
        // Verify specific modifiers
        let always_include = blocks[0].modifiers.iter().find(|(k, _)| k == "always_include").map(|(_, v)| v);
        assert_eq!(always_include, Some(&"true".to_string()));
        
        let priority = blocks[0].modifiers.iter().find(|(k, _)| k == "priority").map(|(_, v)| v);
        assert_eq!(priority, Some(&"8".to_string()));
        
        let order = blocks[0].modifiers.iter().find(|(k, _)| k == "order").map(|(_, v)| v);
        assert_eq!(order, Some(&"0.5".to_string()));
        
        let weight = blocks[0].modifiers.iter().find(|(k, _)| k == "weight").map(|(_, v)| v);
        assert_eq!(weight, Some(&"0.7".to_string()));
        
        let summarize = blocks[0].modifiers.iter().find(|(k, _)| k == "summarize").map(|(_, v)| v);
        assert_eq!(summarize, Some(&"brief".to_string()));
    }
    
    #[test]
    #[ignore]
    fn test_debugging_modifiers() {
        let input = r#"[question name:debug-question debug:true verbosity:high]
What is the meaning of life?
[/question]"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "question");
        assert_eq!(blocks[0].name, Some("debug-question".to_string()));
        
        // Verify specific modifiers
        let debug = blocks[0].modifiers.iter().find(|(k, _)| k == "debug").map(|(_, v)| v);
        assert_eq!(debug, Some(&"true".to_string()));
        
        let verbosity = blocks[0].modifiers.iter().find(|(k, _)| k == "verbosity").map(|(_, v)| v);
        assert_eq!(verbosity, Some(&"high".to_string()));
    }
    
    #[test]
    #[ignore]
    fn test_multiple_modifiers_spacing() {
        let input = r#"[data name:spacing-test format:json priority:10 weight:0.5 order:0.1]
{ "test": "spacing" }
[/data]"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, Some("spacing-test".to_string()));
        
        // Verify all modifiers are correctly parsed despite different spacing
        let format = blocks[0].modifiers.iter().find(|(k, _)| k == "format").map(|(_, v)| v);
        assert_eq!(format, Some(&"json".to_string()));
        
        let priority = blocks[0].modifiers.iter().find(|(k, _)| k == "priority").map(|(_, v)| v);
        assert_eq!(priority, Some(&"10".to_string()));
        
        let weight = blocks[0].modifiers.iter().find(|(k, _)| k == "weight").map(|(_, v)| v);
        assert_eq!(weight, Some(&"0.5".to_string()));
        
        let order = blocks[0].modifiers.iter().find(|(k, _)| k == "order").map(|(_, v)| v);
        assert_eq!(order, Some(&"0.1".to_string()));
    }
    
    #[test]
    #[ignore]
    fn test_quoted_string_modifiers() {
        let input = r#"[api name:api-test url:"https://api.example.com/data" method:"GET" header:"Authorization: Bearer token"]
// API request
[/api]"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "api");
        assert_eq!(blocks[0].name, Some("api-test".to_string()));
        
        // Verify quoted string modifiers are correctly parsed
        let url = blocks[0].modifiers.iter().find(|(k, _)| k == "url").map(|(_, v)| v);
        assert_eq!(url, Some(&"https://api.example.com/data".to_string()));
        
        let method = blocks[0].modifiers.iter().find(|(k, _)| k == "method").map(|(_, v)| v);
        assert_eq!(method, Some(&"GET".to_string()));
        
        let header = blocks[0].modifiers.iter().find(|(k, _)| k == "header").map(|(_, v)| v);
        assert_eq!(header, Some(&"Authorization: Bearer token".to_string()));
    }
    
    #[test]
    fn test_boolean_modifiers() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::blocks::Block;
        
        let mut block = Block::new("data", Some("boolean-test"), "Some test data with boolean modifiers");
        
        // Add boolean modifiers directly
        block.add_modifier("visible", "true");
        block.add_modifier("editable", "false");
        block.add_modifier("expandable", "true");
        
        // Verify boolean modifiers
        let visible = block.modifiers.iter().find(|(k, _)| k == "visible").map(|(_, v)| v);
        assert_eq!(visible, Some(&"true".to_string()));
        
        let editable = block.modifiers.iter().find(|(k, _)| k == "editable").map(|(_, v)| v);
        assert_eq!(editable, Some(&"false".to_string()));
        
        let expandable = block.modifiers.iter().find(|(k, _)| k == "expandable").map(|(_, v)| v);
        assert_eq!(expandable, Some(&"true".to_string()));
        
        // Verify helper methods
        assert!(block.has_modifier("visible"));
        assert!(block.has_modifier("editable"));
        assert!(block.has_modifier("expandable"));
        
        assert_eq!(block.get_modifier("visible"), Some(&"true".to_string()));
        assert_eq!(block.get_modifier("editable"), Some(&"false".to_string()));
        assert_eq!(block.get_modifier("expandable"), Some(&"true".to_string()));
    }
    
    #[test]
    #[ignore]
    fn test_numeric_modifiers() {
        let input = r#"[visualization name:chart width:800 height:600 margin:10]
Chart configuration
[/visualization]"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "visualization");
        
        // Verify numeric modifiers
        let width = blocks[0].modifiers.iter().find(|(k, _)| k == "width").map(|(_, v)| v);
        assert_eq!(width, Some(&"800".to_string()));
        
        let height = blocks[0].modifiers.iter().find(|(k, _)| k == "height").map(|(_, v)| v);
        assert_eq!(height, Some(&"600".to_string()));
        
        let margin = blocks[0].modifiers.iter().find(|(k, _)| k == "margin").map(|(_, v)| v);
        assert_eq!(margin, Some(&"10".to_string()));
    }
}
