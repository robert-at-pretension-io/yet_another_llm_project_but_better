use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;
use std::path::{Path, PathBuf};

// Import necessary modules from your project
// You may need to adjust these imports based on your actual project structure
use yet_another_llm_project_but_better::parser::parse_document;
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
use yet_another_llm_project_but_better::file_watcher::{FileWatcher, FileEvent, FileEventType};

#[test]
fn test_file_watcher_detects_new_blocks() {
    // Create a temporary directory that will be automatically cleaned up
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path_string = temp_dir.path().join("test_document.md").to_string_lossy().to_string();
    let file_path_string_clone = file_path_string.clone();
    let file_path = Path::new(&file_path_string_clone);
    
    // Create initial file with some content but no blocks
    let initial_content = "# Test Document\n\nThis is a test document with no blocks initially.\n";
    fs::write(file_path, initial_content).expect("Failed to write initial content");
    
    // Create a channel to receive file events
    let (sender, receiver) = std::sync::mpsc::channel();
    
    // Create a shared executor that will be used to track executed blocks
    let executor = Arc::new(Mutex::new(MetaLanguageExecutor::new()));
    let executor_clone = Arc::clone(&executor);
    
    // Create and start the file watcher
    let mut watcher = FileWatcher::new_with_sender(sender);
    watcher.watch(file_path)
        .expect("Failed to start watching file");
    
    // Create a flag to track when we've detected the new blocks
    let detected = Arc::new(Mutex::new(false));
    let detected_clone = Arc::clone(&detected);
    
    // Spawn a thread to handle file events
    let _handle = thread::spawn(move || {
        for event in receiver {
            if let FileEvent { path, event_type: FileEventType::Modified } = event {
                if path == file_path_string {
                    // Read the updated file content
                    let content = fs::read_to_string(&path).expect("Failed to read file");
                    
                    // Parse the document to find blocks
                    match parse_document(&content) {
                        Ok(blocks) => {
                            let mut exec = executor_clone.lock().unwrap();
                            
                            // Process each block
                            for block in blocks {
                                if let Some(name) = &block.name {
                                    // Add the block to the executor
                                    exec.blocks.insert(name.clone(), block.clone());
                                    
                                    // If we found the expected block, set the detected flag
                                    if name == "example" {
                                        let mut detected = detected_clone.lock().unwrap();
                                        *detected = true;
                                        break;
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Error parsing document: {}", e);
                        }
                    }
                }
            }
        }
    });
    
    // Wait a moment for the watcher to initialize
    thread::sleep(Duration::from_millis(100));
    
    // Now modify the file to add a block
    let updated_content = "# Test Document\n\n<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<meta:document xmlns:meta=\"https://example.com/meta-language\">\n<meta:code language=\"python\" name=\"example\">\n<![CDATA[\nprint(\"Hello, world!\")\n]]>\n</meta:code>\n</meta:document>\n";
    fs::write(file_path, updated_content).expect("Failed to write updated content");
    
    // Wait for the watcher to detect the change and process it
    thread::sleep(Duration::from_millis(500));
    
    // Check if we detected the block
    let detected = Arc::clone(&detected);
    let was_detected = *detected.lock().unwrap();
    assert!(was_detected, "Block was not detected by the file watcher");
}

// This test is mainly focused on the FileWatcher mechanics, not the actual block
// execution, so we'll use a simpler approach
#[test]
fn test_file_watcher_detects_modified_blocks() {
    // Create a temporary directory that will be automatically cleaned up
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path_string = temp_dir.path().join("test_document_modified.md").to_string_lossy().to_string();
    let file_path_string_clone = file_path_string.clone();
    let file_path = Path::new(&file_path_string_clone);
    
    // Create initial file with some content and a block
    let initial_content = "# Test Document\n\n<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<meta:document xmlns:meta=\"https://example.com/meta-language\">\n<meta:code language=\"python\" name=\"example\">\n<![CDATA[\nprint(\"Initial version\")\n]]>\n</meta:code>\n</meta:document>\n";
    fs::write(file_path, initial_content).expect("Failed to write initial content");
    
    // Create a channel to receive file events
    let (sender, receiver) = std::sync::mpsc::channel();
    
    // Create a flag to track when we've detected the modified block
    let modified = Arc::new(Mutex::new(false));
    let modified_clone = Arc::clone(&modified);
    
    // Create and start the file watcher
    let mut watcher = FileWatcher::new_with_sender(sender);
    watcher.watch(file_path)
        .expect("Failed to start watching file");
    
    // Spawn a thread to handle file events
    let _handle = thread::spawn(move || {
        for event in receiver {
            if let FileEvent { path, event_type: FileEventType::Modified } = event {
                if path == file_path_string {
                    // Read the updated file content
                    let content = fs::read_to_string(&path).expect("Failed to read file");
                    
                    // Check if the content contains the modified version
                    if content.contains("Modified version") {
                        let mut modified = modified_clone.lock().unwrap();
                        *modified = true;
                    }
                }
            }
        }
    });
    
    // Wait a moment for the watcher to initialize
    thread::sleep(Duration::from_millis(100));
    
    // Now modify the file to update the block
    let updated_content = "# Test Document\n\n<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<meta:document xmlns:meta=\"https://example.com/meta-language\">\n<meta:code language=\"python\" name=\"example\">\n<![CDATA[\nprint(\"Modified version\")\n]]>\n</meta:code>\n</meta:document>\n";
    fs::write(file_path, updated_content).expect("Failed to write updated content");
    
    // Wait for the watcher to detect the change and process it
    thread::sleep(Duration::from_millis(500));
    
    // Check if we detected the modified block
    let modified = Arc::clone(&modified);
    let was_modified = *modified.lock().unwrap();
    assert!(was_modified, "Block modification was not detected by the file watcher");
}