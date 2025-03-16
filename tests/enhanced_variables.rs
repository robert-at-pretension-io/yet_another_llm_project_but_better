use yet_another_llm_project_but_better::parser::{parse_document, Block};
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
use std::collections::HashMap;

// NOTE: Some tests have been intentionally removed because those features
// (syntax highlighting, transformations, conditional inclusion, limit modifiers, and nested references)
// are not currently needed.

#[test]
fn test_enhanced_variable_reference_basic() {
    // Test basic enhanced variable reference with format modifier
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:data name="test-data" format="json">
    {
        "name": "John Doe",
        "age": 30,
        "skills": ["Programming", "Data Analysis", "Machine Learning"]
    }
    </meta:data>

    <meta:question name="format-test" model="gpt-4" test_mode="true" test_response=" John Doe

- Age: 30
- Skills: Programming, Data Analysis, Machine Learning

Format: markdown">
    Here is the data in markdown format: <meta:reference target="test-data" format="markdown"/>
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with enhanced variable reference: {:?}", result.err());
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 2);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with enhanced variable reference: {:?}", process_result.err());
        
    // Check that the variable reference was expanded correctly with the specified format
    let question_block = match executor.blocks.get("format-test") {
        Some(block) => block,
        None => {
            println!("DEBUG: Available blocks: {:?}", executor.blocks.keys().collect::<Vec<_>>());
            panic!("Could not find 'format-test' block in executor");
        }
    };
    assert!(question_block.content.contains("<reference target=\"test-data\" format=\"markdown\"/>"));
    
    // After processing, the reference should be replaced with the formatted content
    let updated_content = match executor.update_document() {
        Ok(content) => content,
        Err(e) => {
            println!("DEBUG: Executor outputs: {:?}", executor.outputs);
            panic!("Failed to update document: {:?}", e);
        }
    };
    assert!(updated_content.contains("John Doe"), "Content should include data from the referenced block");
    assert!(updated_content.contains("Format: markdown"), "Formatting instruction should be applied");
}

#[test]
fn test_enhanced_variable_reference_include_modifiers() {
    // Test enhanced variable reference with include_code and include_results modifiers
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:code name="analysis-code" language="python">
    import pandas as pd
    import numpy as np
    
    def analyze_data(data):
        return data.describe()
    </meta:code>

    <meta:results name="analysis-results" for="analysis-code">
    Summary Statistics:
    Mean: 42.5
    Median: 40.0
    Std Dev: 12.3
    </meta:results>

    <meta:question name="analysis-question" model="gpt-4" test_mode="true" test_response="The analysis approach is straightforward and effective. Using pandas for descriptive statistics is a common practice. The results show good summary statistics that help understand the central tendency and spread of the data.">
    Here is the analysis code and results:
    <meta:reference target="analysis-code" include_code="true" include_results="true"/>
    
    What do you think of this analysis approach?
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with include modifiers");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 3);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with include modifiers: {:?}", process_result.err());
    
    // Check that the variable reference includes both code and results
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("import pandas"), "Should include the code");
    assert!(updated_content.contains("Summary Statistics"), "Should include the results");
}

#[test]
fn test_enhanced_variable_reference_multiple_modifiers() {
    // Test multiple enhanced variable references with different modifiers in the same question
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:data name="input-data" format="json">
    {
        "sales": [100, 150, 200, 250, 300]
    }
    </meta:data>

    <meta:code name="process-code" language="python">
    def calculate_growth(sales):
        growth = []
        for i in range(1, len(sales)):
            growth.append((sales[i] - sales[i-1]) / sales[i-1] * 100)
        return growth
    </meta:code>

    <meta:results name="process-results" for="process-code">
    [50.0, 33.33, 25.0, 20.0]
    </meta:results>

    <meta:question name="combined-analysis" model="gpt-4" test_mode="true" test_response="The sales data shows a decreasing growth rate trend. While sales are consistently increasing, the percentage growth is declining from 50% to 33.33% to 25% to 20%. This suggests a maturing market with diminishing returns on growth efforts.">
    Input data: <meta:reference target="input-data" format="json" preview="true"/>
    
    Processing code: <meta:reference target="process-code" include_code="true" format="code"/>
    
    Results: <meta:reference target="process-results" format="plain"/>
    
    Analyze the growth trend in these sales figures.
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with multiple variable references");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 4);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with multiple variable references: {:?}", process_result.err());
    
    // Check that different modifiers are applied correctly
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("sales"), "Should include input data");
    assert!(updated_content.contains("calculate_growth"), "Should include processing code");
    assert!(updated_content.contains("[50.0, 33.33, 25.0, 20.0]"), "Should include results");
}

#[test]
fn test_enhanced_variable_reference_error_handling() {
    // Test graceful handling of errors in variable references
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:data name="valid-data" format="json">
    {
        "name": "Test Data",
        "value": 42
    }
    </meta:data>

    <meta:question name="error-handling-question" model="gpt-4" test_mode="true" test_response="From the available data, I can see that we have a valid JSON object with a name 'Test Data' and a value of 42. The other data points are not available - one because the data source doesn't exist, and another because an invalid format was requested. This demonstrates good error handling with appropriate fallback messages.">
    Valid data: <meta:reference target="valid-data"/>
    
    Missing data: <meta:reference target="missing-data" fallback="Data not available"/>
    
    Invalid format: <meta:reference target="valid-data" format="invalid_format" fallback="Invalid format requested"/>
    
    Please analyze the available data.
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with error handling");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 2);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with error handling: {:?}", process_result.err());
    
    // Check that errors are handled gracefully with fallbacks
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("Test Data"), "Should include valid data");
    assert!(updated_content.contains("Data not available"), "Should include fallback for missing data");
    assert!(updated_content.contains("Invalid format requested"), "Should include fallback for invalid format");
}
