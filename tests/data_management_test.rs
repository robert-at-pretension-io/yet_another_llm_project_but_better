#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::Block;
    
    #[test]
    fn test_data_block_json() {
        // Create a data block manually
        let mut block = Block::new("data", Some("user-profile"), "");
        block.add_modifier("format", "json");
        block.content = r#"{
  "name": "John Doe",
  "age": 30,
  "email": "john.doe@example.com",
  "preferences": {
    "theme": "dark",
    "notifications": true
  }
}"#.to_string();
        
        assert_eq!(block.block_type, "data");
        assert_eq!(block.name, Some("user-profile".to_string()));
        assert_eq!(block.get_modifier("format"), Some(&"json".to_string()));
        
        let content = block.content.as_str();
        assert!(content.contains("John Doe"));
        assert!(content.contains("preferences"));
        assert!(content.contains("dark"));
    }
    
    #[test]
    fn test_data_block_csv() {
        // Create a data block manually
        let mut block = Block::new("data", Some("sales-data"), "");
        block.add_modifier("format", "csv");
        block.content = r#"date,product,quantity,revenue
2023-01-15,Widget A,120,1200.00
2023-01-16,Widget B,85,1700.00
2023-01-17,Widget A,95,950.00
2023-01-18,Widget C,50,1500.00
2023-01-19,Widget B,75,1500.00"#.to_string();
        
        assert_eq!(block.block_type, "data");
        assert_eq!(block.name, Some("sales-data".to_string()));
        assert_eq!(block.get_modifier("format"), Some(&"csv".to_string()));
        
        let content = block.content.as_str();
        assert!(content.contains("date,product,quantity,revenue"));
        assert!(content.contains("Widget A"));
        assert!(content.contains("1500.00"));
    }
    
    #[test]
    fn test_variable_block() {
        // Create a variable block manually
        let block = Block::new("variable", Some("api-base-url"), "https://api.example.com/v2");
        
        assert_eq!(block.block_type, "variable");
        assert_eq!(block.name, Some("api-base-url".to_string()));
        assert_eq!(block.content, "https://api.example.com/v2");
    }
    
    #[test]
    fn test_secret_block() {
        // Create a secret block manually
        let mut block = Block::new("secret", Some("api-key"), "// This is a placeholder, the actual value will be loaded from environment");
        block.add_modifier("env", "EXAMPLE_API_KEY");
        
        assert_eq!(block.block_type, "secret");
        assert_eq!(block.name, Some("api-key".to_string()));
        assert_eq!(block.get_modifier("env"), Some(&"EXAMPLE_API_KEY".to_string()));
    }
    
    #[test]
    fn test_filename_block() {
        // Create a filename block manually
        let mut block = Block::new("filename", Some("config-file"), "// This block will include the content of the specified file");
        block.add_modifier("path", "./config/settings.json");
        
        assert_eq!(block.block_type, "filename");
        assert_eq!(block.name, Some("config-file".to_string()));
        assert_eq!(block.get_modifier("path"), Some(&"./config/settings.json".to_string()));
    }
    
    #[test]
    fn test_memory_block() {
        // Create a memory block manually
        let block = Block::new("memory", Some("conversation-history"), "This block will persist across sessions and store the conversation history.");
        
        assert_eq!(block.block_type, "memory");
        assert_eq!(block.name, Some("conversation-history".to_string()));
        assert!(block.content.contains("persist across sessions"));
    }
    
    #[test]
    fn test_variable_references() {
        // Create blocks manually
        let base_url_block = Block::new("variable", Some("base-url"), "https://api.example.com");
        let endpoint_block = Block::new("variable", Some("endpoint"), "/users");
        
        let mut api_request_block = Block::new("code:javascript", Some("api-request"), "");
        api_request_block.content = r#"const url = '${base-url}${endpoint}';
fetch(url)
  .then(response => response.json())
  .then(data => console.log(data));"#.to_string();
        
        let blocks = vec![base_url_block, endpoint_block, api_request_block];
        
        assert_eq!(blocks.len(), 3);
        
        assert_eq!(blocks[0].name, Some("base-url".to_string()));
        assert_eq!(blocks[1].name, Some("endpoint".to_string()));
        assert_eq!(blocks[2].name, Some("api-request".to_string()));
        
        // Check if the code block contains variable references
        let code_content = blocks[2].content.as_str();
        assert!(code_content.contains("${base-url}${endpoint}"));
    }
}
