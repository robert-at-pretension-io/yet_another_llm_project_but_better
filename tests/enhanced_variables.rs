use yet_another_llm_project_but_better::parser::{parse_document, Block};
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
use std::collections::HashMap;

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
    Here is the data in markdown format: ${test-data:format=markdown}
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
    assert!(question_block.content.contains("${test-data:format=markdown}"));
    
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
    ${analysis-code:include_code=true,include_results=true}
    
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
    Input data: ${input-data:format=json,preview=true}
    
    Processing code: ${process-code:include_code=true,format=code}
    
    Results: ${process-results:format=plain}
    
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
fn test_enhanced_variable_reference_limit_modifier() {
    // Test enhanced variable reference with limit modifier to truncate content
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:data name="large-dataset" format="csv">
id,name,value
1,item1,100
2,item2,200
3,item3,300
4,item4,400
5,item5,500
6,item6,600
7,item7,700
8,item8,800
9,item9,900
10,item10,1000
    </meta:data>

    <meta:question name="limited-data-question" model="gpt-4" test_mode="true" test_response="I observe a linear pattern in the data. The ID increases by 1 for each row, and the value increases by 100 for each row. This suggests a direct proportional relationship between ID and value.">
    Here is a preview of the dataset with only the first 5 lines:
    ${large-dataset:limit=5}
    
    What pattern do you observe in the data?
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with limit modifier");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 2);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with limit modifier: {:?}", process_result.err());
    
    // Check that the content is limited to the specified number of lines
    let updated_content = executor.update_document().unwrap();
    println!("DEBUG TEST: Updated content: {}", updated_content);
    
    // Extract just the question block content to check the limited variable reference
    let question_start = updated_content.find("<meta:question name=\"limited-data-question\"").unwrap();
    let content_start = updated_content[question_start..].find(">").unwrap() + question_start + 1;
    let content_end = updated_content[content_start..].find("</meta:question>").unwrap() + content_start;
    let question_content = &updated_content[content_start..content_end];
    
    println!("DEBUG TEST: Question content: {}", question_content);
    
    // Now check the extracted question content
    assert!(question_content.contains("item1"), "Question should include first items");
    assert!(question_content.contains("item5"), "Question should include item5");
    assert!(!question_content.contains("item6"), "Question should not include item6 or beyond");
    assert!(question_content.contains("...(truncated)"), "Question should include truncation indicator");
}

#[test]
fn test_enhanced_variable_reference_conditional_inclusion() {
    // Test conditional inclusion based on variable values
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:variable name="show-details" format="plain">true</meta:variable>
    
    <meta:data name="user-data" format="json">
    {
        "name": "Alice Smith",
        "email": "alice@example.com",
        "sensitive_info": {
            "ssn": "123-45-6789",
            "account": "ACC123456"
        }
    }
    </meta:data>

    <meta:question name="conditional-question" model="gpt-4" test_mode="true" test_response="The user profile is for Alice Smith who can be contacted at alice@example.com. The profile includes sensitive information such as SSN (123-45-6789) and account number (ACC123456).">
    User profile:
    ${user-data:include_sensitive=${show-details}}
    
    Summarize the user profile.
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with conditional inclusion");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 3);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with conditional inclusion: {:?}", process_result.err());
    
    // Check that sensitive information is included based on the condition
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("Alice Smith"), "Should include basic user data");
    assert!(updated_content.contains("123-45-6789"), "Should include sensitive info because show-details is true");
    
    // Now change the variable value and verify that sensitive info is excluded
    let mut variable_block = executor.blocks.get("show-details").unwrap().clone();
    variable_block.content = "false".to_string();
    executor.blocks.insert("show-details".to_string(), variable_block);
    
    // Reprocess with updated variable
    executor.process_document(input).unwrap();
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("Alice Smith"), "Should still include basic user data");
    assert!(!updated_content.contains("123-45-6789"), "Should not include sensitive info because show-details is false");
}

#[test]
fn test_enhanced_variable_reference_transformation() {
    // Test transformation modifiers that change the content
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:data name="raw-text" format="plain">
    This is some example text with UPPERCASE and lowercase words.
    It also includes 123 numbers and special characters like @#$%.
    </meta:data>

    <meta:question name="transform-question" model="gpt-4" test_mode="true" test_response="The text transformations demonstrate different ways to manipulate text data. The uppercase transformation converts all characters to capital letters, making it useful for standardization or emphasis. The lowercase transformation does the opposite, converting all to small letters, which is helpful for case-insensitive comparisons. The substring transformation extracts only the first 20 characters, which can be useful for previews or when working with fixed-width fields.">
    Original text: ${raw-text}
    
    Uppercase: ${raw-text:transform=uppercase}
    Lowercase: ${raw-text:transform=lowercase}
    
    First 20 chars: ${raw-text:transform=substring(0,20)}
    
    Analyze these different text transformations.
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with transformation modifiers");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 2);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with transformation modifiers: {:?}", process_result.err());
    
    // Check that transformations are applied correctly
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("THIS IS SOME EXAMPLE"), "Should include uppercase transformation");
    assert!(updated_content.contains("this is some example"), "Should include lowercase transformation");
    assert!(updated_content.contains("This is some examp"), "Should include substring transformation");
}

#[test]
fn test_enhanced_variable_reference_highlighting() {
    // Test syntax highlighting enhancement
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:code name="example-python" language="python">
    def factorial(n):
        if n <= 1:
            return 1
        return n * factorial(n-1)
    
    result = factorial(5)
    print(f"5! = {result}")
    </meta:code>

    <meta:code name="example-sql" language="sql">
    SELECT 
        users.name,
        COUNT(orders.id) as order_count
    FROM users
    JOIN orders ON users.id = orders.user_id
    GROUP BY users.id
    HAVING COUNT(orders.id) > 5
    ORDER BY order_count DESC;
    </meta:code>

    <meta:question name="highlight-question" model="gpt-4" test_mode="true" test_response="The Python code implements a recursive factorial function that calculates 5! (5 factorial) which equals 120. It demonstrates recursion with a base case (n <= 1) and a recursive case.

The SQL query retrieves users and counts their orders, filtering to only show users with more than 5 orders. It demonstrates joins, aggregation with GROUP BY, filtering aggregated results with HAVING, and sorting with ORDER BY.">
    Python code with syntax highlighting:
    ${example-python:highlight=true}
    
    SQL query with syntax highlighting:
    ${example-sql:highlight=true}
    
    Explain both code examples.
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with highlighting modifiers");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 3);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with highlighting modifiers: {:?}", process_result.err());
    
    // Check that syntax highlighting markers are added
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("```python"), "Should include Python syntax highlighting marker");
    assert!(updated_content.contains("```sql"), "Should include SQL syntax highlighting marker");
    assert!(updated_content.contains("```"), "Should include code block markers");
}

#[test]
fn test_enhanced_variable_reference_nesting() {
    // Test nested variable references (one variable reference contains another)
    let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:variable name="format-type" format="plain">markdown</meta:variable>
    
    <meta:data name="user-stats" format="json">
    {
        "visits": 127,
        "purchases": 12,
        "total_spent": 345.67
    }
    </meta:data>

    <meta:template name="user-template" format="markdown">
    ## User Statistics
    
    - **Visits**: ${user-stats.visits}
    - **Purchases**: ${user-stats.purchases}
    - **Total Spent**: $${user-stats.total_spent}
    
    Conversion Rate: ${user-stats.purchases / user-stats.visits * 100}%
    </meta:template>

    <meta:question name="nested-question" model="gpt-4" test_mode="true" test_response="Based on the user statistics, this appears to be a relatively engaged customer with a conversion rate of about 9.4%. With 127 visits resulting in 12 purchases, they show consistent interest in the products. The average purchase value is approximately $28.81 ($345.67 total spent divided by 12 purchases), which suggests they may be buying mid-range items rather than high-ticket products.">
    User data in ${format-type} format:
    
    ${user-template:format=${format-type}}
    
    What insights can you derive from this user's behavior?
    </meta:question>
</meta:document>"#;

    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with nested variable references");
    
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 4);
    
    // Create executor and process document
    let mut executor = MetaLanguageExecutor::new();
    let process_result = executor.process_document(input);
    assert!(process_result.is_ok(), "Failed to process document with nested variable references: {:?}", process_result.err());
    
    // Check that nested references are resolved correctly
    let updated_content = executor.update_document().unwrap();
    assert!(updated_content.contains("## User Statistics"), "Should include template header");
    assert!(updated_content.contains("Visits: 127"), "Should include visits from user-stats");
    assert!(updated_content.contains("Conversion Rate"), "Should include calculated conversion rate");
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
    Valid data: ${valid-data}
    
    Missing data: ${missing-data:fallback="Data not available"}
    
    Invalid format: ${valid-data:format=invalid_format,fallback="Invalid format requested"}
    
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
