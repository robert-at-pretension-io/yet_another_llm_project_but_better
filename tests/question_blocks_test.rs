use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

use metalang::{
    executor::MetaLanguageExecutor,
    file_watcher::{FileWatcher, FileEvent, FileEventType},
    llm_client::{LlmClient, LlmRequestConfig, LlmProvider},
    parser::{parse_document, Block},
};

// Mock implementation of LlmClient for testing
struct MockLlmClient;

impl MockLlmClient {
    fn mock_response(_prompt: &str) -> String {
        "This is a mock LLM response for testing purposes.".to_string()
    }
}

#[test]
fn test_question_block_with_file_watcher() {
    // Create a temporary directory for our test file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_question.md");
    
    // Create a file with a question block
    let initial_content = r#"# Test Question Block

[question name:test-question model:gpt-3.5-turbo]
What is the capital of France?
[/question]
"#;
    
    fs::write(&file_path, initial_content).expect("Failed to write test file");
    
    // Set up file watcher
    let (sender, receiver) = mpsc::channel();
    let mut watcher = FileWatcher::new(sender).expect("Failed to create file watcher");
    
    // Start watching the file
    watcher.watch(file_path.to_string_lossy().to_string()).expect("Failed to watch file");
    
    // Create executor with mocked LLM client
    let mut executor = MetaLanguageExecutor::new();
    
    // Patch the executor to use our mock LLM client
    // In a real implementation, we would inject the mock client
    // For this test, we'll rely on the executor's fallback mechanism
    
    // Add a fallback for the question block
    executor.fallbacks.insert(
        "test-question".to_string(),
        MockLlmClient::mock_response("What is the capital of France?".to_string())
    );
    
    // Modify the file to trigger the watcher
    let modified_content = r#"# Test Question Block

[question name:test-question model:gpt-3.5-turbo]
What is the capital of France? And why is it famous?
[/question]
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
    
    // Parse the document
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    let blocks = parse_document(&content).expect("Failed to parse document");
    
    // Find the question block
    let question_block = blocks.iter()
        .find(|b| b.block_type == "question" && b.name.as_deref() == Some("test-question"))
        .expect("Question block not found");
    
    // Execute the question block
    let result = executor.execute_block("test-question");
    
    // Verify the result
    assert!(result.is_ok(), "Failed to execute question block: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.contains("mock LLM response"), "Unexpected response: {}", output);
    
    // Clean up
    drop(watcher); // Stop the file watcher
    temp_dir.close().expect("Failed to clean up temp directory");
}
