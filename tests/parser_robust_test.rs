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
    
    [error_results name:test-error-results for:test-