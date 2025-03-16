#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
    
    #[test]
    fn test_parse_basic_data_block() {
        let mut block = Block::new("data", Some("test-data"), r#"{"value": 42}"#);
        block.add_modifier("format", "json");
        
        let blocks = vec![block];
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[0].name, Some("test-data".to_string()));
        
        // Check if the modifier is correctly parsed
        let has_format_json = blocks[0].modifiers.iter()
            .any(|(key, value)| key == "format" && value == "json");
        assert!(has_format_json);
        
        // Check content
        assert!(blocks[0].content.contains("42"));
    }
    
    #[test]
    fn test_parse_code_block() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:code:python name="example-code">
  <![CDATA[
print("Hello, world!")
  ]]>
  </meta:code:python>
</meta:document>"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "code:python");
        assert_eq!(blocks[0].name, Some("example-code".to_string()));
        assert_eq!(blocks[0].content.trim(), r#"print("Hello, world!")"#);
    }
    
    #[test]
    fn test_parse_multiple_blocks() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="test-data" format="json">
  <![CDATA[
{"key": "value"}
  ]]>
  </meta:data>
  
  <meta:code:python name="process-data">
  <![CDATA[
import json
data = json.loads('''${test-data}''')
print(data)
  ]]>
  </meta:code:python>
</meta:document>"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[1].block_type, "code:python");
    }
    
    #[test]
    fn test_parse_template_declaration() {
        let mut block = Block::new("template", Some("data-processor"), "");
        block.add_modifier("model", "gpt-4");
        block.content = r#"[code:python name:process-${dataset-name}]
import json
data = json.loads('''${dataset}''')
[/code:python]"#.to_string();
        
        let blocks = vec![block];
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "template");
        assert_eq!(blocks[0].name, Some("data-processor".to_string()));
        
        // Check if model modifier is correctly parsed
        let has_model_gpt4 = blocks[0].modifiers.iter()
            .any(|(key, value)| key == "model" && value == "gpt-4");
        assert!(has_model_gpt4);
    }
    

    
    #[test]
    fn test_executor_is_cacheable() {
        let executor = MetaLanguageExecutor::new();
        
        let mut block = Block::new("code:python", Some("test-block"), "print('hello')");
        block.add_modifier("cache_result", "true");
        
        assert!(executor.is_cacheable(&block));
        
        let mut block2 = Block::new("code:python", Some("test-block2"), "print('hello')");
        block2.add_modifier("cache_result", "false");
        
        assert!(!executor.is_cacheable(&block2));
    }
}
