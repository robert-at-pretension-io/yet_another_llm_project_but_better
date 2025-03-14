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
    
    #[test]
    fn test_section_with_data_block() {
        let input = r#"[section:data-container name:sample-data]
# Sample Data Section

[data name:numbers]
[1, 2, 3, 4, 5]
[/data]

This section contains a data block.
[/section:data-container]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with data block: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 section block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section:data-container");
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
        let input = r#"[section:code-example name:python-example]
# Python Example

[code:python name:hello-world]
def hello():
    print("Hello, World!")

hello()
[/code:python]

This section demonstrates a Python code block.
[/section:code-example]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with code block: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 section block, found {}", blocks.len());
        
        let section = &blocks[0];
        assert_eq!(section.block_type, "section:code-example");
        assert_eq!(section.name, Some("python-example".to_string()));
        assert!(section.content.contains("# Python Example"));
        
        // Check that the section has 1 child code block
        assert_eq!(section.children.len(), 1, "Expected 1 child code block, found {}", section.children.len());
        
        // Check the code block
        let code_block = &section.children[0];
        assert_eq!(code_block.block_type, "code:python");
        assert_eq!(code_block.name, Some("hello-world".to_string()));
        assert!(code_block.content.contains("def hello():"));
        assert!(code_block.content.contains("print(\"Hello, World!\")"));
    }
    
    #[test]
    fn test_deeply_nested_sections() {
        let input = r#"[section:document name:nested-doc]
# Deeply Nested Document

[section:chapter name:chapter1]
## Chapter 1

[section:subsection name:subsection1]
### Subsection 1.1

This is the deepest level.
[/section:subsection]

Back to chapter level.
[/section:chapter]

Document conclusion.
[/section:document]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse deeply nested sections: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        // Check the document section
        let document = &blocks[0];
        assert_eq!(document.block_type, "section:document");
        assert_eq!(document.name, Some("nested-doc".to_string()));
        assert!(document.content.contains("# Deeply Nested Document"));
        assert!(document.content.contains("Document conclusion."));
        
        // Check that the document has 1 child chapter
        assert_eq!(document.children.len(), 1, "Expected 1 child chapter, found {}", document.children.len());
        
        // Check the chapter section
        let chapter = &document.children[0];
        assert_eq!(chapter.block_type, "section:chapter");
        assert_eq!(chapter.name, Some("chapter1".to_string()));
        assert!(chapter.content.contains("## Chapter 1"));
        assert!(chapter.content.contains("Back to chapter level."));
        
        // Check that the chapter has 1 child subsection
        assert_eq!(chapter.children.len(), 1, "Expected 1 child subsection, found {}", chapter.children.len());
        
        // Check the subsection
        let subsection = &chapter.children[0];
        assert_eq!(subsection.block_type, "section:subsection");
        assert_eq!(subsection.name, Some("subsection1".to_string()));
        assert!(subsection.content.contains("### Subsection 1.1"));
        assert!(subsection.content.contains("This is the deepest level."));
    }
    
    #[test]
    fn test_sections_with_mixed_blocks() {
        let input = r#"[section:report name:mixed-content]
# Mixed Content Report

[data name:metrics]
{"users": 100, "views": 500, "conversions": 25}
[/data]

## Analysis

[code:python name:analysis-script]
import json
data = json.loads(metrics)
conversion_rate = data["conversions"] / data["users"] * 100
print(f"Conversion rate: {conversion_rate}%")
[/code:python]

[section:conclusion name:report-conclusion]
Based on the analysis, we can see that the conversion rate is 25%.
[/section:conclusion]

[/section:report]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse section with mixed blocks: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        // Check the report section
        let report = &blocks[0];
        assert_eq!(report.block_type, "section:report");
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
        assert_eq!(code_block.block_type, "code:python");
        assert_eq!(code_block.name, Some("analysis-script".to_string()));
        assert!(code_block.content.contains("import json"));
        
        // Check the conclusion section (third child)
        let conclusion = &report.children[2];
        assert_eq!(conclusion.block_type, "section:conclusion");
        assert_eq!(conclusion.name, Some("report-conclusion".to_string()));
        assert!(conclusion.content.contains("Based on the analysis"));
    }
}
