#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
    
    #[test]
    fn test_parse_xml_data_block() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="test-data" format="json">
  <![CDATA[
  {"value": 42}
  ]]>
  </meta:data>
</meta:document>"#;
        
        let blocks = parse_document(input).unwrap();
        
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
}
