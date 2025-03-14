#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::parse_document;
    
    #[test]
    fn test_basic_section() {
        let input = r#"[section:chapter name:introduction]
This is an introduction chapter.
[/section:chapter]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse basic section: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 section block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section:chapter");
        assert_eq!(section.name, Some("introduction".to_string()));
        assert_eq!(section.content, "This is an introduction chapter.");
    }
    
    #[test]
    fn test_multiple_sections() {
        let input = r#"[section:chapter name:introduction]
This is an introduction chapter.
[/section:chapter]

[section:chapter name:methodology]
This is the methodology chapter.
[/section:chapter]

[section:chapter name:conclusion]
This is the conclusion chapter.
[/section:chapter]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse multiple sections: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 3, "Expected 3 section blocks, found {}", blocks.len());
        
        assert_eq!(blocks[0].block_type, "section:chapter");
        assert_eq!(blocks[0].name, Some("introduction".to_string()));
        
        assert_eq!(blocks[1].block_type, "section:chapter");
        assert_eq!(blocks[1].name, Some("methodology".to_string()));
        
        assert_eq!(blocks[2].block_type, "section:chapter");
        assert_eq!(blocks[2].name, Some("conclusion".to_string()));
    }
    
    #[test]
    fn test_nested_sections() {
        let input = r#"[section:document name:research-paper]
# Research Paper

[section:chapter name:introduction]
This is an introduction chapter.
[/section:chapter]

[section:chapter name:conclusion]
This is the conclusion chapter.
[/section:chapter]

[/section:document]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse nested sections: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        // Check the document section
        let document = &blocks[0];
        assert_eq!(document.block_type, "section:document");
        assert_eq!(document.name, Some("research-paper".to_string()));
        assert!(document.content.contains("# Research Paper"));
        
        // Check that the document has 2 child sections
        assert_eq!(document.children.len(), 2, "Expected 2 child sections, found {}", document.children.len());
        
        // Check the introduction section (first child)
        let intro = &document.children[0];
        assert_eq!(intro.block_type, "section:chapter");
        assert_eq!(intro.name, Some("introduction".to_string()));
        assert_eq!(intro.content, "This is an introduction chapter.");
        
        // Check the conclusion section (second child)
        let conclusion = &document.children[1];
        assert_eq!(conclusion.block_type, "section:chapter");
        assert_eq!(conclusion.name, Some("conclusion".to_string()));
        assert_eq!(conclusion.content, "This is the conclusion chapter.");
    }
}
