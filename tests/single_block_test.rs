#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{MetaLanguageParser, Rule, parse_document};
    use pest::Parser;
    
    #[test]
    fn test_direct_block_parsing() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:data name="test-data" format="json">
<![CDATA[
{"value": 42}
]]>
</meta:data>
</meta:document>"#;
        
        // Parse as a document
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok(), "Failed to parse document: {:?}", doc_result.err());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 data block, found {}", blocks.len());
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[0].name, Some("test-data".to_string()));
        
        // Check modifiers
        let format = blocks[0].get_modifier("format");
        assert_eq!(format, Some(&"json".to_string()));
    }
    
    #[test]
    fn test_code_block_parsing() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:code language="python" name="calculate-sum">
<![CDATA[
def add(a, b):
    return a + b

result = add(5, 7)
print(result)
]]>
</meta:code>
</meta:document>"#;
        
        // Parse as a document
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok(), "Failed to parse code block in document: {:?}", doc_result.err());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 code block, found {}", blocks.len());
        assert_eq!(blocks[0].block_type, "code:python");
        assert_eq!(blocks[0].name, Some("calculate-sum".to_string()));
    }
    
    #[test]
    fn test_shell_block_parsing() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:shell name="install-deps">
<![CDATA[
pip install pandas numpy matplotlib
]]>
</meta:shell>
</meta:document>"#;
        
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok(), "Failed to parse shell block: {:?}", doc_result.err());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks[0].block_type, "shell");
        assert_eq!(blocks[0].name, Some("install-deps".to_string()));
        assert_eq!(blocks[0].content.trim(), "pip install pandas numpy matplotlib");
    }
    
    #[test]
    fn test_variable_block_parsing() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:variable name="api-key">
<![CDATA[
sk-1234567890abcdef
]]>
</meta:variable>
</meta:document>"#;
        
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok(), "Failed to parse variable block: {:?}", doc_result.err());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks[0].block_type, "variable");
        assert_eq!(blocks[0].name, Some("api-key".to_string()));
        assert_eq!(blocks[0].content.trim(), "sk-1234567890abcdef");
    }
    
    #[test]
    fn test_block_with_modifiers() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:data name="config" format="json">
<![CDATA[
{"server": "localhost", "port": 8080}
]]>
</meta:data>
</meta:document>"#;
        
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok(), "Failed to parse document: {:?}", doc_result.err());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[0].name, Some("config".to_string()));
        
        // Check modifiers
        let format = blocks[0].get_modifier("format");
        assert_eq!(format, Some(&"json".to_string()));
    }
}
