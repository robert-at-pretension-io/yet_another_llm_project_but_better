#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    
    #[test]
    fn test_simple_section() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:section type="test" name="simple-section">
    This is a simple section with no child blocks.
  </meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse simple section: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section");
        assert_eq!(section.get_modifier("type").unwrap(), "test");
        assert_eq!(section.name, Some("simple-section".to_string()));
        assert!(section.content.contains("This is a simple section"));
        assert_eq!(section.children.len(), 0, "Expected 0 child blocks, found {}", section.children.len());
    }
    
    #[test]
    fn test_section_with_one_child() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:section type="test" name="parent">
    Some content before the child.

    <meta:data name="child-data">
    test data
    </meta:data>

    Some content after the child.
  </meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with child: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section");
        assert_eq!(section.get_modifier("type").unwrap(), "test");
        assert_eq!(section.name, Some("parent".to_string()));
        assert!(section.content.contains("Some content before"));
        assert!(section.content.contains("Some content after"));
        
        // Check that the section has 1 child block
        assert_eq!(section.children.len(), 1, "Expected 1 child block, found {}", section.children.len());
        
        // Check the data block (child)
        let data_block = &section.children[0];
        assert_eq!(data_block.block_type, "data");
        assert_eq!(data_block.name, Some("child-data".to_string()));
        assert!(data_block.content.contains("test data"));
    }
    
    #[test]
    fn test_section_with_multiple_children() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:section type="document" name="multi-child">
    # Header

    <meta:data name="first-child">
    data content
    </meta:data>

    <meta:code language="python" name="second-child">
    print("Hello")
    </meta:code>

    <meta:shell name="third-child">
    echo "Test"
    </meta:shell>

  </meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with multiple children: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section");
        assert_eq!(section.get_modifier("type").unwrap(), "document");
        assert_eq!(section.name, Some("multi-child".to_string()));
        assert!(section.content.contains("# Header"));
        
        // Check that the section has 3 child blocks
        assert_eq!(section.children.len(), 3, "Expected 3 child blocks, found {}", section.children.len());
        
        // Check the first child
        let first_child = &section.children[0];
        assert_eq!(first_child.block_type, "data");
        assert_eq!(first_child.name, Some("first-child".to_string()));
        
        // Check the second child
        let second_child = &section.children[1];
        assert_eq!(second_child.block_type, "code");
        assert_eq!(second_child.get_modifier("language").unwrap(), "python");
        assert_eq!(second_child.name, Some("second-child".to_string()));
        
        // Check the third child
        let third_child = &section.children[2];
        assert_eq!(third_child.block_type, "shell");
        assert_eq!(third_child.name, Some("third-child".to_string()));
    }
    
    #[test]
    fn test_nested_sections() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:section type="outer" name="parent">
    Outer content

    <meta:section type="inner" name="child">
    Inner content
    </meta:section>

    More outer content
  </meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse nested sections: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        let outer = &blocks[0];
        assert_eq!(outer.block_type, "section");
        assert_eq!(outer.get_modifier("type").unwrap(), "outer");
        assert_eq!(outer.name, Some("parent".to_string()));
        assert!(outer.content.contains("Outer content"));
        assert!(outer.content.contains("More outer content"));
        
        // Check that the outer section has 1 child block
        assert_eq!(outer.children.len(), 1, "Expected 1 child block, found {}", outer.children.len());
        
        // Check the inner section
        let inner = &outer.children[0];
        assert_eq!(inner.block_type, "section");
        assert_eq!(inner.get_modifier("type").unwrap(), "inner");
        assert_eq!(inner.name, Some("child".to_string()));
        assert!(inner.content.contains("Inner content"));
    }
}
