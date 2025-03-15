use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

// Import necessary modules from your project
// You may need to adjust these imports based on your actual project structure
use yet_another_llm_project_but_better::parser::{parse_document, Block};
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
use yet_another_llm_project_but_better::file_watcher::{FileWatcher, FileEvent, FileEventType};

#[test]
fn test_file_watcher_detects_new_blocks() {
    // Create a temporary directory that will be automatically cleaned up
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_document.md");
    
    // Create initial file with some content but no blocks
    let initial_content = "# Test Document\n\nThis is a test document with no blocks initially.\n";
    fs::write(&file_path, initial_content).expect("Failed to write initial content");
    
    // Create a channel to receive file events
    let (sender, receiver) = std::sync::mpsc::channel();
    
    // Create a shared executor that will be used to track executed blocks
    let executor = Arc::new(Mutex::new(MetaLanguageExecutor::new()));
    let executor_clone = Arc::clone(&executor);
    
    // Create and start the file watcher
    let mut watcher = FileWatcher::new(sender);
    watcher.watch(file_path.to_str().unwrap().to_string())
        .expect("Failed to start watching file");
    
    // Create a flag to track when we've detected the new blocks
    let detected = Arc::new(Mutex::new(false));
    let detected_clone = Arc::clone(&detected);
    
    // Clone the file path before moving it into the closure
    let file_path_clone = file_path.clone();
    
    // Spawn a thread to handle file events
    let _handle = thread::spawn(move || {
        for event in receiver {
            if let FileEvent { path, event_type: FileEventType::Modified } = event {
                if path == file_path_clone.to_str().unwrap() {
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
                                    
                                    // If we found our test block, mark as detected
                                    if name == "test-python-block" {
                                        let mut detected = detected_clone.lock().unwrap();
                                        *detected = true;
                                    }
                                }
                            }
                        },
                        Err(e) => println!("Error parsing document: {:?}", e),
                    }
                }
            }
        }
    });
    
    // Wait a moment for the watcher to start
    thread::sleep(Duration::from_millis(100));
    
    // Now modify the file to add a new block
    let updated_content = format!("{}\n\
        [code:python name:test-python-block]\n\
        print('Hello from Python!')\n\
        [/code]\n", initial_content);
    
    // Write the updated content
    fs::write(&file_path, updated_content).expect("Failed to write updated content");
    
    // Wait for the watcher to detect the change (with timeout)
    let mut attempts = 0;
    while attempts < 50 {
        {
            let detected = detected.lock().unwrap();
            if *detected {
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
        attempts += 1;
    }
    
    // Verify that the block was detected and added to the executor
    {
        let exec = executor.lock().unwrap();
        assert!(exec.blocks.contains_key("test-python-block"), 
                "The test-python-block was not detected by the file watcher");
    }
    
    // Clean up (tempdir will automatically clean up the file)
    drop(watcher); // Stop the watcher
}

#[test]
fn test_file_watcher_detects_modified_blocks() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_document_modified.md");
    
    // Create initial file with a block
    let initial_content = "# Test Document\n\n\
        [code:python name:modify-me]\n\
        print('Initial content')\n\
        [/code]\n";
    fs::write(&file_path, initial_content).expect("Failed to write initial content");
    
    // Create a channel to receive file events
    let (sender, receiver) = std::sync::mpsc::channel();
    
    // Create a shared executor
    let executor = Arc::new(Mutex::new(MetaLanguageExecutor::new()));
    let executor_clone = Arc::clone(&executor);
    
    // Initialize the executor with the initial block
    {
        let content = fs::read_to_string(&file_path).expect("Failed to read file");
        if let Ok(blocks) = parse_document(&content) {
            let mut exec = executor.lock().unwrap();
            for block in blocks {
                if let Some(name) = &block.name {
                    exec.blocks.insert(name.clone(), block);
                }
            }
        }
    }
    
    // Create a flag to track when we've detected the modified block
    let modified = Arc::new(Mutex::new(false));
    let modified_clone = Arc::clone(&modified);
    
    // Create and start the file watcher
    let mut watcher = FileWatcher::new(sender);
    watcher.watch(file_path.to_str().unwrap().to_string())
        .expect("Failed to start watching file");
    
    // Clone the file path before moving it into the closure
    let file_path_clone = file_path.clone();
    
    // Spawn a thread to handle file events
    let _handle = thread::spawn(move || {
        for event in receiver {
            if let FileEvent { path, event_type: FileEventType::Modified } = event {
                if path == file_path_clone.to_str().unwrap() {
                    // Read the updated file content
                    let content = fs::read_to_string(&path).expect("Failed to read file");
                    
                    // Parse the document to find blocks
                    if let Ok(blocks) = parse_document(&content) {
                        let mut exec = executor_clone.lock().unwrap();
                        
                        // Process each block
                        for block in blocks {
                            if let Some(name) = &block.name {
                                // Update the block in the executor
                                if name == "modify-me" && block.content.contains("Modified content") {
                                    exec.blocks.insert(name.clone(), block);
                                    let mut modified = modified_clone.lock().unwrap();
                                    *modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    
    // Wait a moment for the watcher to start
    thread::sleep(Duration::from_millis(100));
    
    // Now modify the file to change the block
    let updated_content = "# Test Document\n\n\
        [code:python name:modify-me]\n\
        print('Modified content')\n\
        [/code]\n";
    
    // Write the updated content
    fs::write(&file_path, updated_content).expect("Failed to write updated content");
    
    // Wait for the watcher to detect the change (with timeout)
    let mut attempts = 0;
    while attempts < 50 {
        {
            let modified = modified.lock().unwrap();
            if *modified {
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
        attempts += 1;
    }
    
    // Verify that the block was modified in the executor
    {
        let exec = executor.lock().unwrap();
        if let Some(block) = exec.blocks.get("modify-me") {
            assert!(block.content.contains("Modified content"), 
                    "The block content was not updated");
        } else {
            panic!("The modify-me block was not found in the executor");
        }
    }
    
    // Clean up
    drop(watcher);
}
