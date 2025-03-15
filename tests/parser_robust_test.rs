//! Parser robustness tests
//! 
//! This file contains comprehensive tests for the Meta Language parser,
//! focusing on edge cases, complex structures, and error handling.

use yet_another_llm_project_but_better::parser::{parse_document, Block, ParserError, utils::extractors::extract_variable_references};
use std::collections::HashSet;

/// Test all block types defined in the language specification
#[test]
fn test_all_block_types() {
    // Create a document with every block type
    let input = r#"
    [question name:test-question]
    What is the meaning of life?
    [/question]
    
    [response name:test-response]
    The meaning of life is 42.
    [/response]
    
    [code:python name:test-code-python]
    print("Hello from Python!")
    [/code:python]
    
    [shell name:test-shell]
    echo "Hello from the shell!"
    [/shell]
    
    [api name:test-api method:GET]
    https://jsonplaceholder.typicode.com/todos/1
    [/api]
    
    [data name:test-data format:json]
    {"key": "value", "number": 42}
    [/data]
    
    [variable name:test-variable]
    sample value
    [/variable]
    
    [secret name:test-secret]
    API_KEY
    [/secret]
    
    [filename name:test-filename]
    test_file.txt
    [/filename]
    
    [results name:test-results for:test-code-python]
    Hello from Python!
    [/results]
    
    [error_results name:test-error-results for:test-#[test]
fn test_complex_document() {
    // Test parsing a complex document with multiple block types and nesting
    let input = r#"
[section:intro name:document_intro]
# Introduction

This is a complex document with multiple block types and nesting.

[code:python name:setup_code]
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

def setup_environment():
    print("Setting up environment")
    return {"ready": True}
[/code:python]

[variable name:config]
{
    "data_source": "example.csv",
    "max_rows": 1000,
    "columns": ["id", "name", "value"]
}
[/variable]

[section:data_processing name:data_section]
## Data Processing

[code:python name:process_data depends:setup_code]
def process_data(source="${config.data_source}"):
    print(f"Processing data from {source}")
    # Use the setup from the previous block
    env = setup_environment()
    if env["ready"]:
        return {"processed": True}
[/code:python]

[data name:sample_data]
{"id": 1, "name": "Example", "value": 42}
[/data]

[shell name:run_script]
python -c "import json; print(json.dumps(${sample_data}))"
[/shell]

[results for:run_script]
{"id": 1, "name": "Example", "value": 42}
[/results]
[/section:data_processing]

[section:visualization name:viz_section]
## Visualization

[code:python name:create_viz depends:process_data]
def create_visualization(data):
    print("Creating visualization")
    # This would normally create a plot
    return "visualization.png"
[/code:python]

[visualization name:data_viz type:bar data:sample_data]
// Visualization configuration
[/visualization]

[preview for:data_viz]
[Bar chart showing sample data]
[/preview]
[/section:visualization]

[template name:report_template]
# ${title}

Data processed: ${data_processed}
Visualization: ${visualization_path}

## Summary
${summary}
[/template]

[template:report_template name:final_report 
  title:"Analysis Report"
  data_processed:"Yes"
  visualization_path:"visualization.png"
  summary:"This is a summary of the analysis."
]
[/template:report_template]

[conditional if:config.max_rows>500]
This section only appears if max_rows is greater than 500.
[/conditional]

[error_results for:missing_block]
Error: Block not found
[/error_results]
[/section:intro]
"#;

    let blocks = parse_document(input).expect("Failed to parse complex document");
    
    // Find the main section
    let intro_section = find_block_by_name(&blocks, "document_intro").expect("Intro section not found");
    assert_eq!(intro_section.block_type, "section:intro");
    
    // Check that the section has the expected number of child blocks
    assert!(intro_section.children.len() >= 5, "Expected at least 5 child blocks, got {}", intro_section.children.len());
    
    // Check nested sections
    let data_section = find_child_by_name(intro_section, "data_section").expect("Data section not found");
    assert_eq!(data_section.block_type, "section:data_processing");
    assert_eq!(data_section.children.len(), 4, "Expected 4 child blocks in data section, got {}", data_section.children.len());
    
    let viz_section = find_child_by_name(intro_section, "viz_section").expect("Visualization section not found");
    assert_eq!(viz_section.block_type, "section:visualization");
    assert_eq!(viz_section.children.len(), 3, "Expected 3 child blocks in viz section, got {}", viz_section.children.len());
    
    // Check variable references
    let process_data = find_child_by_name(data_section, "process_data").expect("Process data block not found");
    assert!(process_data.content.contains("${config.data_source}"));
    
    // Check dependencies
    assert!(has_modifier(process_data, "depends", "setup_code"));
    
    let create_viz = find_child_by_name(viz_section, "create_viz").expect("Create viz block not found");
    assert!(has_modifier(create_viz, "depends", "process_data"));
    
    // Check template invocation
    let final_report = find_block_by_name(&blocks, "final_report").expect("Final report not found");
    assert!(has_modifier(final_report, "title", "Analysis Report"));
    assert!(has_modifier(final_report, "data_processed", "Yes"));
    assert!(has_modifier(final_report, "visualization_path", "visualization.png"));
    assert!(has_modifier(final_report, "summary", "This is a summary of the analysis."));
    
    // Check conditional block
    let conditional = intro_section.children.iter()
        .find(|b| b.block_type == "conditional")
        .expect("Conditional block not found");
    assert!(has_modifier(conditional, "if", "config.max_rows>500"));
}
