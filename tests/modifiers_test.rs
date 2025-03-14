#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::parse_document;
    
    #[test]
    fn test_execution_control_modifiers() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::Block;
        
        let mut block = Block::new("code:python", Some("test-exec"), "print(\"This is a test with execution modifiers\")");
        
        // Add execution control modifiers directly
        block.add_modifier("cache_result", "true");
        block.add_modifier("timeout", "30");
        block.add_modifier("retry", "3");
        block.add_modifier("fallback", "test-fallback");
        block.add_modifier("async", "true");
        
        // Verify execution control modifiers
        let cache_result = block.modifiers.iter().find(|(k, _)| k == "cache_result").map(|(_, v)| v);
        assert_eq!(cache_result, Some(&"true".to_string()));
        
        let timeout = block.modifiers.iter().find(|(k, _)| k == "timeout").map(|(_, v)| v);
        assert_eq!(timeout, Some(&"30".to_string()));
        
        let retry = block.modifiers.iter().find(|(k, _)| k == "retry").map(|(_, v)| v);
        assert_eq!(retry, Some(&"3".to_string()));
        
        let fallback = block.modifiers.iter().find(|(k, _)| k == "fallback").map(|(_, v)| v);
        assert_eq!(fallback, Some(&"test-fallback".to_string()));
        
        let async_mod = block.modifiers.iter().find(|(k, _)| k == "async").map(|(_, v)| v);
        assert_eq!(async_mod, Some(&"true".to_string()));
        
        // Verify helper methods
        assert!(block.has_modifier("cache_result"));
        assert!(block.has_modifier("timeout"));
        assert!(block.has_modifier("retry"));
        assert!(block.has_modifier("fallback"));
        assert!(block.has_modifier("async"));
        
        assert_eq!(block.get_modifier("cache_result"), Some(&"true".to_string()));
        assert_eq!(block.get_modifier("timeout"), Some(&"30".to_string()));
        assert_eq!(block.get_modifier("retry"), Some(&"3".to_string()));
        assert_eq!(block.get_modifier("fallback"), Some(&"test-fallback".to_string()));
        assert_eq!(block.get_modifier("async"), Some(&"true".to_string()));
    }
    
    #[test]
    fn test_context_management_modifiers() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::Block;
        
        let mut block = Block::new("data", Some("context-data"), "{\n  \"context\": \"This is context data\"\n}");
        
        // Add context management modifiers directly
        block.add_modifier("always_include", "true");
        block.add_modifier("priority", "8");
        block.add_modifier("order", "0.5");
        block.add_modifier("weight", "0.7");
        block.add_modifier("summarize", "brief");
        
        // Verify context management modifiers
        let always_include = block.modifiers.iter().find(|(k, _)| k == "always_include").map(|(_, v)| v);
        assert_eq!(always_include, Some(&"true".to_string()));
        
        let priority = block.modifiers.iter().find(|(k, _)| k == "priority").map(|(_, v)| v);
        assert_eq!(priority, Some(&"8".to_string()));
        
        let order = block.modifiers.iter().find(|(k, _)| k == "order").map(|(_, v)| v);
        assert_eq!(order, Some(&"0.5".to_string()));
        
        let weight = block.modifiers.iter().find(|(k, _)| k == "weight").map(|(_, v)| v);
        assert_eq!(weight, Some(&"0.7".to_string()));
        
        let summarize = block.modifiers.iter().find(|(k, _)| k == "summarize").map(|(_, v)| v);
        assert_eq!(summarize, Some(&"brief".to_string()));
        
        // Verify helper methods
        assert!(block.has_modifier("always_include"));
        assert!(block.has_modifier("priority"));
        assert!(block.has_modifier("order"));
        assert!(block.has_modifier("weight"));
        assert!(block.has_modifier("summarize"));
        
        assert_eq!(block.get_modifier("always_include"), Some(&"true".to_string()));
        assert_eq!(block.get_modifier("priority"), Some(&"8".to_string()));
        assert_eq!(block.get_modifier("order"), Some(&"0.5".to_string()));
        assert_eq!(block.get_modifier("weight"), Some(&"0.7".to_string()));
        assert_eq!(block.get_modifier("summarize"), Some(&"brief".to_string()));
    }
    
    #[test]
    fn test_debugging_modifiers() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::Block;
        
        let mut block = Block::new("question", Some("debug-question"), "What is the meaning of life?");
        
        // Add debugging modifiers directly
        block.add_modifier("debug", "true");
        block.add_modifier("verbosity", "high");
        
        // Verify debugging modifiers
        let debug = block.modifiers.iter().find(|(k, _)| k == "debug").map(|(_, v)| v);
        assert_eq!(debug, Some(&"true".to_string()));
        
        let verbosity = block.modifiers.iter().find(|(k, _)| k == "verbosity").map(|(_, v)| v);
        assert_eq!(verbosity, Some(&"high".to_string()));
        
        // Verify helper methods
        assert!(block.has_modifier("debug"));
        assert!(block.has_modifier("verbosity"));
        
        assert_eq!(block.get_modifier("debug"), Some(&"true".to_string()));
        assert_eq!(block.get_modifier("verbosity"), Some(&"high".to_string()));
    }
    
    #[test]
    
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
    fn test_quoted_string_modifiers() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::Block;
        
        let mut block = Block::new("api", Some("api-test"), "// API request");
        
        // Add quoted string modifiers directly
        block.add_modifier("url", "https://api.example.com/data");
        block.add_modifier("method", "GET");
        block.add_modifier("header", "Authorization: Bearer token");
        
        // Verify quoted string modifiers
        let url = block.modifiers.iter().find(|(k, _)| k == "url").map(|(_, v)| v);
        assert_eq!(url, Some(&"https://api.example.com/data".to_string()));
        
        let method = block.modifiers.iter().find(|(k, _)| k == "method").map(|(_, v)| v);
        assert_eq!(method, Some(&"GET".to_string()));
        
        let header = block.modifiers.iter().find(|(k, _)| k == "header").map(|(_, v)| v);
        assert_eq!(header, Some(&"Authorization: Bearer token".to_string()));
        
        // Verify helper methods
        assert!(block.has_modifier("url"));
        assert!(block.has_modifier("method"));
        assert!(block.has_modifier("header"));
        
        assert_eq!(block.get_modifier("url"), Some(&"https://api.example.com/data".to_string()));
        assert_eq!(block.get_modifier("method"), Some(&"GET".to_string()));
        assert_eq!(block.get_modifier("header"), Some(&"Authorization: Bearer token".to_string()));
    }
    
    #[test]
    fn test_boolean_modifiers() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::Block;
        
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
    fn test_numeric_modifiers() {
        // Create block directly
        use yet_another_llm_project_but_better::parser::Block;
        
        let mut block = Block::new("visualization", Some("chart"), "Chart configuration");
        
        // Add numeric modifiers directly
        block.add_modifier("width", "800");
        block.add_modifier("height", "600");
        block.add_modifier("margin", "10");
        
        // Verify numeric modifiers
        let width = block.modifiers.iter().find(|(k, _)| k == "width").map(|(_, v)| v);
        assert_eq!(width, Some(&"800".to_string()));
        
        let height = block.modifiers.iter().find(|(k, _)| k == "height").map(|(_, v)| v);
        assert_eq!(height, Some(&"600".to_string()));
        
        let margin = block.modifiers.iter().find(|(k, _)| k == "margin").map(|(_, v)| v);
        assert_eq!(margin, Some(&"10".to_string()));
        
        // Verify helper methods
        assert!(block.has_modifier("width"));
        assert!(block.has_modifier("height"));
        assert!(block.has_modifier("margin"));
        
        assert_eq!(block.get_modifier("width"), Some(&"800".to_string()));
        assert_eq!(block.get_modifier("height"), Some(&"600".to_string()));
        assert_eq!(block.get_modifier("margin"), Some(&"10".to_string()));
    }
}
