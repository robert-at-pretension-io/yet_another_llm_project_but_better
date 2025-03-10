#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{MetaLanguageParser, Rule};
    use pest::Parser;
    
    #[test]
    fn test_direct_block_parsing() {
        let input = r#"[data name:test-data format:json]
{"value": 42}
[/data]"#;
        
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
}
