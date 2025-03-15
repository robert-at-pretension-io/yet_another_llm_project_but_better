use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

use yet_another_llm_project_but_better::{
    executor::MetaLanguageExecutor,
    file_watcher::{FileWatcher, FileEvent, FileEventType},
    parser::parse_document,
};

#[test]
fn test_question_block_with_test_mode() {
    // Create a temporary directory for our test file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_question.md");
    
    // Create a file with a question block that uses test_mode
    // This will use the simulated response path in the executor
    let initial_content = r#"# Test Question Block

<?xml version="1.0" encoding="UTF-8"?>
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
        executor.blocks.insert(block.name.clone().unwrap_or_default(), block);
    }
    
    // Execute the question block
    let result = executor.execute_block("test-question");
    
    // Verify the result
    assert!(result.is_ok(), "Failed to execute question block: {:?}", result.err());
    let output = result.unwrap();
    assert!(!output.is_empty(), "Response should not be empty");
    assert!(output.contains("This is a simulated response") || 
            output.contains("test mode"), 
            "Expected test mode response, got: {}", output);
    
    // Test file watcher integration
    let (sender, receiver) = mpsc::channel();
    let mut watcher = FileWatcher::new(sender);
    
    // Start watching the file
    watcher.watch(file_path.to_string_lossy().to_string()).expect("Failed to watch file");
    
    // Modify the file to trigger the watcher
    let modified_content = r#"# Test Question Block

<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question name="test-question" test_mode="true">
  What is the capital of France? And why is it famous?
  </meta:question>
</meta:document>
"#;
    
    // Wait a moment to ensure the watcher is ready
    thread::sleep(Duration::from_millis(100));
    
    // Write the modified content
    fs::write(&file_path, modified_content).expect("Failed to update test file");
    
    // Wait for the file event
    let event = receiver.recv_timeout(Duration::from_secs(5))
        .expect("Timed out waiting for file event");
    
    // Verify the event
    assert_eq!(event.event_type, FileEventType::Modified);
    assert_eq!(event.path, file_path.to_string_lossy().to_string());
    
    // Re-parse the document and update the executor
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    let blocks = parse_document(&content).expect("Failed to parse document");
    
    // Clear existing blocks and register the updated ones
    executor.blocks.clear();
    for block in blocks {
        executor.blocks.insert(block.name.clone().unwrap_or_default(), block);
    }
    
    // Execute the question block again
    let result = executor.execute_block("test-question");
    
    // Verify the updated result
    assert!(result.is_ok(), "Failed to execute updated question block: {:?}", result.err());
    let output = result.unwrap();
    assert!(!output.is_empty(), "Response should not be empty");
    
    // Clean up
    drop(watcher); // Stop the file watcher
    temp_dir.close().expect("Failed to clean up temp directory");
}

#[test]
fn test_question_block_fallback() {
    // Create a temporary directory for our test file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_fallback.md");
    
    // Create a file with a question block that has a fallback
    let content = r#"# Test Question Block with Fallback

<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question name="test-fallback" test_mode="true" fallback="Default answer when no response is available">
  What happens if there's no response block?
  </meta:question>
</meta:document>
"#;
    
    fs::write(&file_path, content).expect("Failed to write test file");
    
    // Create executor
    let mut executor = MetaLanguageExecutor::new();
    
    // Parse the document and register blocks with the executor
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    let blocks = parse_document(&content).expect("Failed to parse document");
    
    // Register all blocks with the executor
    for block in blocks {
        executor.blocks.insert(block.name.clone().unwrap_or_default(), block);
    }
    
    // Execute the question block
    let result = executor.execute_block("test-fallback");
    
    // Verify the fallback is used
    assert!(result.is_ok(), "Failed to execute question block: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.contains("Default answer when no response is available"), 
            "Fallback not used: {}", output);
    
    // Clean up
    temp_dir.close().expect("Failed to clean up temp directory");
}

#[test]
fn test_question_block_with_model_parameter() {
    // Create a temporary directory for our test file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_model_param.md");
    
    // Create a file with a question block that specifies a model
    let content = r#"# Test Question Block with Model Parameter

<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question name="test-model-param" test_mode="true" model="gpt-4">
  What is the meaning of life?
  </meta:question>
</meta:document>
"#;
    
    fs::write(&file_path, content).expect("Failed to write test file");
    
    // Create executor
    let mut executor = MetaLanguageExecutor::new();
    
    // Parse the document and register blocks with the executor
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    let blocks = parse_document(&content).expect("Failed to parse document");
    
    // Register all blocks with the executor
    for block in blocks {
        executor.blocks.insert(block.name.clone().unwrap_or_default(), block);
    }
    
    // Execute the question block
    let result = executor.execute_block("test-model-param");
    
    // Verify the result
    assert!(result.is_ok(), "Failed to execute question block: {:?}", result.err());
    let output = result.unwrap();
    assert!(!output.is_empty(), "Response should not be empty");
    
    // Clean up
    temp_dir.close().expect("Failed to clean up temp directory");
}
