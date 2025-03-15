#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::parse_document;
    
    #[test]
    fn test_basic_section() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:section type="chapter" name="introduction">
This is an introduction chapter.
</meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse basic section: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 section block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section");
        assert_eq!(section.name, Some("introduction".to_string()));
        assert_eq!(section.content, "This is an introduction chapter.");
    }
    
    #[test]
    fn test_multiple_sections() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:section type="chapter" name="introduction">
This is an introduction chapter.
</meta:section>

<meta:section type="chapter" name="methodology">
This is the methodology chapter.
</meta:section>

<meta:section type="chapter" name="conclusion">
This is the conclusion chapter.
</meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse multiple sections: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 3, "Expected 3 section blocks, found {}", blocks.len());
        
        assert_eq!(blocks[0].block_type, "section");
        assert_eq!(blocks[0].name, Some("introduction".to_string()));
        assert!(blocks[0].modifiers.iter().any(|(k, v)| k == "type" && v == "chapter"));
        
        assert_eq!(blocks[1].block_type, "section");
        assert_eq!(blocks[1].name, Some("methodology".to_string()));
        assert!(blocks[1].modifiers.iter().any(|(k, v)| k == "type" && v == "chapter"));
        
        assert_eq!(blocks[2].block_type, "section");
        assert_eq!(blocks[2].name, Some("conclusion".to_string()));
        assert!(blocks[2].modifiers.iter().any(|(k, v)| k == "type" && v == "chapter"));
    }
    
    #[test]
    fn test_nested_sections() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:section type="document" name="research-paper">
# Research Paper

<meta:section type="chapter" name="introduction">
This is an introduction chapter.
</meta:section>

<meta:section type="chapter" name="conclusion">
This is the conclusion chapter.
</meta:section>

</meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse nested sections: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        // Check the document section
        let document = &blocks[0];
        assert_eq!(document.block_type, "section");
        assert!(document.modifiers.iter().any(|(k, v)| k == "type" && v == "document"));
        assert_eq!(document.name, Some("research-paper".to_string()));
        assert!(document.content.contains("# Research Paper"));
        
        // Check that the document has 2 child sections
        assert_eq!(document.children.len(), 2, "Expected 2 child sections, found {}", document.children.len());
        
        // Check the introduction section (first child)
        let intro = &document.children[0];
        assert_eq!(intro.block_type, "section");
        assert!(intro.modifiers.iter().any(|(k, v)| k == "type" && v == "chapter"));
        assert_eq!(intro.name, Some("introduction".to_string()));
        assert_eq!(intro.content, "This is an introduction chapter.");
        
        // Check the conclusion section (second child)
        let conclusion = &document.children[1];
        assert_eq!(conclusion.block_type, "section");
        assert!(conclusion.modifiers.iter().any(|(k, v)| k == "type" && v == "chapter"));
        assert_eq!(conclusion.name, Some("conclusion".to_string()));
        assert_eq!(conclusion.content, "This is the conclusion chapter.");
    }
    
    #[test]
    fn test_section_with_data_block() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:section type="data-container" name="sample-data">
# Sample Data Section

<meta:data name="numbers">
[1, 2, 3, 4, 5]
</meta:data>

This section contains a data block.
</meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with data block: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 section block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section");
        assert!(section.modifiers.iter().any(|(k, v)| k == "type" && v == "data-container"));
        assert_eq!(section.name, Some("sample-data".to_string()));
        assert!(section.content.contains("# Sample Data Section"));
        assert!(section.content.contains("This section contains a data block."));
        
        // Check that the section has 1 child data block
        assert_eq!(section.children.len(), 1, "Expected 1 child data block, found {}", section.children.len());
        
        // Check the data block
        let data_block = &section.children[0];
        assert_eq!(data_block.block_type, "data");
        assert_eq!(data_block.name, Some("numbers".to_string()));
        assert_eq!(data_block.content, "[1, 2, 3, 4, 5]");
    }
    
    #[test]
    fn test_section_with_code_block() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:section type="code-example" name="python-example">
# Python Example

<meta:code language="python" name="hello-world">
def hello():
    print("Hello, World!")

hello()
</meta:code>

This section demonstrates a Python code block.
</meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with code block: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 section block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section");
        assert!(section.modifiers.iter().any(|(k, v)| k == "type" && v == "code-example"));
        assert_eq!(section.name, Some("python-example".to_string()));
        assert!(section.content.contains("# Python Example"));
        
        // Check that the section has 1 child code block
        assert_eq!(section.children.len(), 1, "Expected 1 child code block, found {}", section.children.len());
        
        // Check the code block
        let code_block = &section.children[0];
        assert_eq!(code_block.block_type, "code");
        assert!(code_block.modifiers.iter().any(|(k, v)| k == "language" && v == "python"));
        assert_eq!(code_block.name, Some("hello-world".to_string()));
        assert!(code_block.content.contains("def hello():"));
        assert!(code_block.content.contains("print(\"Hello, World!\")"));
    }
    
    #[test]
    fn test_deeply_nested_sections() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:section type="document" name="nested-doc">
# Deeply Nested Document

<meta:section type="chapter" name="chapter1">
## Chapter 1

<meta:section type="subsection" name="subsection1">
### Subsection 1.1

This is the deepest level.
</meta:section>

Back to chapter level.
</meta:section>

Document conclusion.
</meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse deeply nested sections: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        // Check the document section
        let document = &blocks[0];
        assert_eq!(document.block_type, "section");
        assert!(document.modifiers.iter().any(|(k, v)| k == "type" && v == "document"));
        assert_eq!(document.name, Some("nested-doc".to_string()));
        assert!(document.content.contains("# Deeply Nested Document"));
        assert!(document.content.contains("Document conclusion."));
        
        // Check that the document has 1 child chapter
        assert_eq!(document.children.len(), 1, "Expected 1 child chapter, found {}", document.children.len());
        
        // Check the chapter section
        let chapter = &document.children[0];
        assert_eq!(chapter.block_type, "section");
        assert!(chapter.modifiers.iter().any(|(k, v)| k == "type" && v == "chapter"));
        assert_eq!(chapter.name, Some("chapter1".to_string()));
        assert!(chapter.content.contains("## Chapter 1"));
        assert!(chapter.content.contains("Back to chapter level."));
        
        // Check that the chapter has 1 child subsection
        assert_eq!(chapter.children.len(), 1, "Expected 1 child subsection, found {}", chapter.children.len());
        
        // Check the subsection
        let subsection = &chapter.children[0];
        assert_eq!(subsection.block_type, "section");
        assert!(subsection.modifiers.iter().any(|(k, v)| k == "type" && v == "subsection"));
        assert_eq!(subsection.name, Some("subsection1".to_string()));
        assert!(subsection.content.contains("### Subsection 1.1"));
        assert!(subsection.content.contains("This is the deepest level."));
    }
    
    #[test]
    fn test_sections_with_mixed_blocks() {
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:section type="report" name="mixed-content">
# Mixed Content Report

<meta:data name="metrics">
{"users": 100, "views": 500, "conversions": 25}
</meta:data>

## Analysis

<meta:code language="python" name="analysis-script">
import json
data = json.loads(metrics)
conversion_rate = data["conversions"] / data["users"] * 100
print(f"Conversion rate: {conversion_rate}%")
</meta:code>

<meta:section type="conclusion" name="report-conclusion">
Based on the analysis, we can see that the conversion rate is 25%.
</meta:section>

</meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with mixed blocks: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        // Check the report section
        let report = &blocks[0];
        assert_eq!(report.block_type, "section");
        assert!(report.modifiers.iter().any(|(k, v)| k == "type" && v == "report"));
        assert_eq!(report.name, Some("mixed-content".to_string()));
        assert!(report.content.contains("# Mixed Content Report"));
        assert!(report.content.contains("## Analysis"));
        
        // Check that the report has 3 child blocks
        assert_eq!(report.children.len(), 3, "Expected 3 child blocks, found {}", report.children.len());
        
        // Check the data block (first child)
        let data_block = &report.children[0];
        assert_eq!(data_block.block_type, "data");
        assert_eq!(data_block.name, Some("metrics".to_string()));
        assert!(data_block.content.contains("\"users\": 100"));
        
        // Check the code block (second child)
        let code_block = &report.children[1];
        assert_eq!(code_block.block_type, "code");
        assert_eq!(code_block.name, Some("analysis-script".to_string()));
        assert!(code_block.content.contains("import json"));
        
        // Check the conclusion section (third child)
        let conclusion = &report.children[2];
        assert_eq!(conclusion.block_type, "section");
        assert!(conclusion.modifiers.iter().any(|(k, v)| k == "type" && v == "conclusion"));
        assert_eq!(conclusion.name, Some("report-conclusion".to_string()));
        assert!(conclusion.content.contains("Based on the analysis"));
    }
}
