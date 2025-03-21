#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
    
    #[test]
    fn test_parse_xml_data_block() {
        let xml_input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="test-data" format="json">
  <![CDATA[
  {"value": 42}
  ]]>
  </meta:data>
</meta:document>"#;
        
        let blocks = parse_document(xml_input).expect("Failed to parse XML document");
        
        // Verify we got one block
        assert_eq!(blocks.len(), 1);
        
        // Check the block type
        assert_eq!(blocks[0].block_type, "data");
        
        // Check the name
        assert_eq!(blocks[0].name, Some("test-data".to_string()));
        
        // Check the format modifier
        let format = blocks[0].get_modifier("format").expect("Format modifier not found");
        assert_eq!(format, "json");
        
        // Check the content
        let expected_content = r#"{"value": 42}"#;
        assert!(blocks[0].content.contains(expected_content));
    }
}
