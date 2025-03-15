use std::fs;
use tempfile::TempDir;
use yet_another_llm_project_but_better::parser::{parse_document, Block};
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;

fn main() {
    // Create a temporary directory for our test file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_question.md");
    
    // Create a file with a question block that uses test_mode
    let initial_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question name="test-question" test_mode="true">
  What is the capital of France?
  </meta:question>
</meta:document>
"#;
    
    fs::write(&file_path, initial_content).expect("Failed to write test file");
    
    // Create executor
    let mut executor = MetaLanguageExecutor::new();
    
    // Parse the document and register blocks with the executor
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    let blocks = parse_document(&content).expect("Failed to parse document");
    
    // Register all blocks with the executor
    for block in blocks {
        if let Some(name) = &block.name {
            executor.blocks.insert(name.clone(), block.clone());
        }
    }
    
    // Execute the question block
    let result = executor.execute_block("test-question");
    
    // Verify the result
    match result {
        Ok(output) => {
            println!("Success! Output: {}", output);
            assert!(!output.is_empty(), "Response should not be empty");
        },
        Err(e) => {
            println!("Error: {:?}", e);
            panic!("Failed to execute question block: {:?}", e);
        }
    }
    
    // Clean up
    temp_dir.close().expect("Failed to clean up temp directory");
    println!("Test completed successfully!");
}
