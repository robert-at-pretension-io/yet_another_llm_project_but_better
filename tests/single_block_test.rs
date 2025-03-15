#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{MetaLanguageParser, Rule, parse_document};
    use pest::Parser;
    
    #[test]
    fn test_direct_block_parsing() {
        let input = r#"<meta:data name="test-data" format="json">
{"value": 42}
</meta:data>"#;
        
        // Try parsing just as a data_block instead of a document
        let pairs = MetaLanguageParser::parse(Rule::data_block, input);
        assert!(pairs.is_ok(), "Failed to parse data block: {:?}", pairs.err());
        
        let pairs = pairs.unwrap();
        let mut pair_count = 0;
        for _ in pairs {
            pair_count += 1;
        }
        
        assert_eq!(pair_count, 1, "Expected 1 data block, found {}", pair_count);
    }
    
    #[test]
    fn test_code_block_parsing() {
        let input = r#"<meta:code language="python" name="calculate-sum">
def add(a, b):
    return a + b

result = add(5, 7)
print(result)
</meta:code>"#;
        
        let pairs = MetaLanguageParser::parse(Rule::code_block, input);
        assert!(pairs.is_ok(), "Failed to parse code block: {:?}", pairs.err());
        
        // Also test as part of a document
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok(), "Failed to parse code block in document: {:?}", doc_result.err());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 code block, found {}", blocks.len());
        assert_eq!(blocks[0].block_type, "code:python");
        assert_eq!(blocks[0].name, Some("calculate-sum".to_string()));
    }
    
    #[test]
    fn test_shell_block_parsing() {
        let input = r#"<meta:shell name="install-deps">
pip install pandas numpy matplotlib
</meta:shell>"#;
        
        let pairs = MetaLanguageParser::parse(Rule::shell_block, input);
        assert!(pairs.is_ok(), "Failed to parse shell block: {:?}", pairs.err());
        
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks[0].block_type, "shell");
        assert_eq!(blocks[0].name, Some("install-deps".to_string()));
        assert_eq!(blocks[0].content.trim(), "pip install pandas numpy matplotlib");
    }
    
    #[test]
    fn test_variable_block_parsing() {
        let input = r#"<meta:variable name="api-key">
sk-1234567890abcdef
</meta:variable>"#;
        
        let pairs = MetaLanguageParser::parse(Rule::variable_block, input);
        assert!(pairs.is_ok(), "Failed to parse variable block: {:?}", pairs.err());
        
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks[0].block_type, "variable");
        assert_eq!(blocks[0].name, Some("api-key".to_string()));
        assert_eq!(blocks[0].content.trim(), "sk-1234567890abcdef");
    }
    
    #[test]
    
    fn test_block_with_modifiers() {
        let input = r#"<meta:data name="config" format="json">
{"server": "localhost", "port": 8080}
</meta:data>"#;
        
        let doc_result = parse_document(input);
        assert!(doc_result.is_ok());
        
        let blocks = doc_result.unwrap();
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[0].name, Some("config".to_string()));
        
        // Check modifiers
        let format = blocks[0].modifiers.iter().find(|(k, _)| k == "format").map(|(_, v)| v);
        assert_eq!(format, Some(&"json".to_string()));
    }
}
