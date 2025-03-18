//! Integration tests for the file watcher functionality.
//!
//! These tests verify that the file watcher correctly detects file system events
//! and processes the appropriate files.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

use anyhow::{Result, anyhow};
use tempfile::tempdir;

use yet_another_llm_project_but_better::file_watcher::{FileWatcher, FileEvent, FileEventType};

/// Helper function to convert String errors to anyhow errors
fn watch(watcher: &mut FileWatcher, path: String) -> Result<()> {
    watcher.watch(path.clone())
        .map_err(|e| anyhow!("Failed to watch path {}: {}", path, e))
}

/// Helper function to convert String errors to anyhow errors
fn unwatch(watcher: &mut FileWatcher, path: &str) -> Result<()> {
    watcher.unwatch(path)
        .map_err(|e| anyhow!("Failed to unwatch path {}: {}", path, e))
}

/// Creates a temporary directory with test files for file watcher tests
fn setup_test_directory() -> Result<(tempfile::TempDir, PathBuf, PathBuf)> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let test_dir = temp_dir.path().to_path_buf();
    
    // Create a test file with the .meta extension
    let meta_file = test_dir.join("test.meta");
    let mut file = File::create(&meta_file)?;
    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(file, "<meta:document xmlns:meta=\"https://example.com/meta-language\">")?;
    writeln!(file, "  <meta:code language=\"python\" name=\"hello\" auto_execute=\"true\">")?;
    writeln!(file, "  <![CDATA[")?;
    writeln!(file, "  print(\"Hello, world!\")")?;
    writeln!(file, "  ]]>")?;
    writeln!(file, "  </meta:code>")?;
    writeln!(file, "</meta:document>")?;
    
    // Create a test file with the .xml extension
    let xml_file = test_dir.join("test.xml");
    let mut file = File::create(&xml_file)?;
    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(file, "<meta:document xmlns:meta=\"https://example.com/meta-language\">")?;
    writeln!(file, "  <meta:data name=\"test-data\" format=\"json\">")?;
    writeln!(file, "  <![CDATA[")?;
    writeln!(file, "  {{\"value\": 42, \"message\": \"Hello, world!\"}}")?;
    writeln!(file, "  ]]>")?;
    writeln!(file, "  </meta:data>")?;
    writeln!(file, "</meta:document>")?;
    
    Ok((temp_dir, meta_file, xml_file))
}

/// Creates a subdirectory with additional test files for the recursive watching test
fn add_test_subdirectory(test_dir: &Path) -> Result<PathBuf> {
    // Create a subdirectory
    let sub_dir = test_dir.join("subdir");
    fs::create_dir(&sub_dir)?;
    
    // Create a test file in the subdirectory
    let sub_file = sub_dir.join("subtest.meta");
    let mut file = File::create(&sub_file)?;
    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(file, "<meta:document xmlns:meta=\"https://example.com/meta-language\">")?;
    writeln!(file, "  <meta:variable name=\"test-var\">")?;
    writeln!(file, "  This is a test variable in a subdirectory")?;
    writeln!(file, "  </meta:variable>")?;
    writeln!(file, "</meta:document>")?;
    
    Ok(sub_file)
}

/// Waits for and collects all file events for a specified duration
fn collect_events(rx: &Receiver<FileEvent>, wait_time: Duration) -> Vec<FileEvent> {
    let mut events = Vec::new();
    let end_time = std::time::Instant::now() + wait_time;
    
    while std::time::Instant::now() < end_time {
        if let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        thread::sleep(Duration::from_millis(50));
    }
    
    events
}

/// Test that the file watcher can be created and destroyed properly
#[test]
fn test_file_watcher_creation() {
    let (tx, _rx) = channel();
    let _watcher = FileWatcher::new_with_sender(tx);
    // The watcher will be dropped at the end of this scope
    // We're testing that no panics occur during creation and destruction
}

/// Test watching a single file for changes
#[test]
fn test_watch_single_file() -> Result<()> {
    let (temp_dir, meta_file, _) = setup_test_directory()?;
    
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new_with_sender(tx);
    
    // Start watching the meta file
    watch(&mut watcher, meta_file.to_string_lossy().to_string())?;
    
    // Allow some time for the watcher to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Modify the meta file
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&meta_file)?;
        
        writeln!(file, "<!-- This is a modification -->")?;
        file.flush()?;
    }
    
    // Wait for and collect events
    let events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we received a modification event for the meta file
    assert!(!events.is_empty(), "No file events received");
    assert!(events.iter().any(|e| 
        e.path == meta_file.to_string_lossy() && 
        e.event_type == FileEventType::Modified
    ), "Did not receive a modification event for the meta file");
    
    // Keep temp_dir alive until the end of the test
    drop(temp_dir);
    
    Ok(())
}

/// Test watching a directory for changes
#[test]
fn test_watch_directory() -> Result<()> {
    let (temp_dir, meta_file, xml_file) = setup_test_directory()?;
    let test_dir = temp_dir.path().to_path_buf();
    
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new_with_sender(tx);
    
    // Start watching the directory
    watch(&mut watcher, test_dir.to_string_lossy().to_string())?;
    
    // Allow some time for the watcher to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Modify the meta file
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&meta_file)?;
        
        writeln!(file, "<!-- This is a modification to the meta file -->")?;
        file.flush()?;
    }
    
    // Wait for and collect events
    let meta_events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we received a modification event for the meta file
    assert!(!meta_events.is_empty(), "No file events received for meta file");
    assert!(meta_events.iter().any(|e| 
        e.path == meta_file.to_string_lossy() && 
        e.event_type == FileEventType::Modified
    ), "Did not receive a modification event for the meta file");
    
    // Modify the xml file
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&xml_file)?;
        
        writeln!(file, "<!-- This is a modification to the xml file -->")?;
        file.flush()?;
    }
    
    // Wait for and collect events
    let xml_events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we received a modification event for the xml file
    assert!(!xml_events.is_empty(), "No file events received for xml file");
    assert!(xml_events.iter().any(|e| 
        e.path == xml_file.to_string_lossy() && 
        e.event_type == FileEventType::Modified
    ), "Did not receive a modification event for the xml file");
    
    // Keep temp_dir alive until the end of the test
    drop(temp_dir);
    
    Ok(())
}

/// Test the recursive watching of directories
#[test]
fn test_recursive_directory_watching() -> Result<()> {
    let (temp_dir, _, _) = setup_test_directory()?;
    let test_dir = temp_dir.path().to_path_buf();
    
    // Create a subdirectory with test files
    let sub_file = add_test_subdirectory(&test_dir)?;
    
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new_with_sender(tx);
    
    // Start watching the main directory (should watch subdirectories recursively)
    watch(&mut watcher, test_dir.to_string_lossy().to_string())?;
    
    // Allow some time for the watcher to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Modify the file in the subdirectory
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&sub_file)?;
        
        writeln!(file, "<!-- This is a modification to the subdirectory file -->")?;
        file.flush()?;
    }
    
    // Wait for and collect events
    let events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we received a modification event for the file in the subdirectory
    assert!(!events.is_empty(), "No file events received");
    assert!(events.iter().any(|e| 
        e.path == sub_file.to_string_lossy() && 
        e.event_type == FileEventType::Modified
    ), "Did not receive a modification event for the file in the subdirectory");
    
    // Keep temp_dir alive until the end of the test
    drop(temp_dir);
    
    Ok(())
}

/// Test creating new files in watched directories
#[test]
fn test_file_creation() -> Result<()> {
    let (temp_dir, _, _) = setup_test_directory()?;
    let test_dir = temp_dir.path().to_path_buf();
    
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new_with_sender(tx);
    
    // Start watching the directory
    watch(&mut watcher, test_dir.to_string_lossy().to_string())?;
    
    // Allow some time for the watcher to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Create a new file
    let new_file = test_dir.join("new_file.meta");
    {
        let mut file = File::create(&new_file)?;
        writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(file, "<meta:document xmlns:meta=\"https://example.com/meta-language\">")?;
        writeln!(file, "  <meta:variable name=\"new-var\">New value</meta:variable>")?;
        writeln!(file, "</meta:document>")?;
        file.flush()?;
    }
    
    // Wait for and collect events
    let events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we received a creation event for the new file
    assert!(!events.is_empty(), "No file events received");
    assert!(events.iter().any(|e| 
        e.path == new_file.to_string_lossy() && 
        e.event_type == FileEventType::Created
    ), "Did not receive a creation event for the new file");
    
    // Keep temp_dir alive until the end of the test
    drop(temp_dir);
    
    Ok(())
}

/// Test file deletion in watched directories
#[test]
fn test_file_deletion() -> Result<()> {
    let (temp_dir, meta_file, _) = setup_test_directory()?;
    let test_dir = temp_dir.path().to_path_buf();
    
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new_with_sender(tx);
    
    // Start watching the directory
    watch(&mut watcher, test_dir.to_string_lossy().to_string())?;
    
    // Allow some time for the watcher to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Delete the meta file
    fs::remove_file(&meta_file)?;
    
    // Wait for and collect events
    let events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we received a deletion event for the meta file
    assert!(!events.is_empty(), "No file events received");
    assert!(events.iter().any(|e| 
        e.path == meta_file.to_string_lossy() && 
        e.event_type == FileEventType::Deleted
    ), "Did not receive a deletion event for the meta file");
    
    // Keep temp_dir alive until the end of the test
    drop(temp_dir);
    
    Ok(())
}

/// Test multiple file watchers watching the same directory
#[test]
fn test_multiple_watchers() -> Result<()> {
    let (temp_dir, meta_file, _) = setup_test_directory()?;
    let test_dir = temp_dir.path().to_path_buf();
    
    // Create two separate watchers
    let (tx1, rx1) = channel();
    let mut watcher1 = FileWatcher::new_with_sender(tx1);
    
    let (tx2, rx2) = channel();
    let mut watcher2 = FileWatcher::new_with_sender(tx2);
    
    // Start watching the directory with both watchers
    watch(&mut watcher1, test_dir.to_string_lossy().to_string())?;
    watch(&mut watcher2, test_dir.to_string_lossy().to_string())?;
    
    // Allow some time for the watchers to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Modify the meta file
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&meta_file)?;
        
        writeln!(file, "<!-- This is a modification -->")?;
        file.flush()?;
    }
    
    // Wait for and collect events from both receivers
    let events1 = collect_events(&rx1, Duration::from_secs(2));
    let events2 = collect_events(&rx2, Duration::from_secs(2));
    
    // Check that both watchers received the event
    assert!(!events1.is_empty(), "No file events received by watcher 1");
    assert!(!events2.is_empty(), "No file events received by watcher 2");
    
    assert!(events1.iter().any(|e| 
        e.path == meta_file.to_string_lossy() && 
        e.event_type == FileEventType::Modified
    ), "Watcher 1 did not receive a modification event");
    
    assert!(events2.iter().any(|e| 
        e.path == meta_file.to_string_lossy() && 
        e.event_type == FileEventType::Modified
    ), "Watcher 2 did not receive a modification event");
    
    // Keep temp_dir alive until the end of the test
    drop(temp_dir);
    
    Ok(())
}

/// Test unwatching a file or directory
#[test]
fn test_unwatch() -> Result<()> {
    let (temp_dir, meta_file, xml_file) = setup_test_directory()?;
    
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new_with_sender(tx);
    
    // Start watching both files
    watch(&mut watcher, meta_file.to_string_lossy().to_string())?;
    watch(&mut watcher, xml_file.to_string_lossy().to_string())?;
    
    // Allow some time for the watcher to initialize
    thread::sleep(Duration::from_millis(500));
    
    // Modify both files to verify they're being watched
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&meta_file)?;
        
        writeln!(file, "<!-- This is a modification to meta file -->")?;
        file.flush()?;
    }
    
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&xml_file)?;
        
        writeln!(file, "<!-- This is a modification to xml file -->")?;
        file.flush()?;
    }
    
    // Wait for and collect events
    let initial_events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we received events for both files
    assert!(initial_events.iter().any(|e| e.path == meta_file.to_string_lossy()), 
           "Did not receive events for meta file");
    assert!(initial_events.iter().any(|e| e.path == xml_file.to_string_lossy()),
           "Did not receive events for xml file");
    
    // Now unwatch the meta file
    unwatch(&mut watcher, &meta_file.to_string_lossy())?;
    
    // Allow some time for the unwatch to take effect
    thread::sleep(Duration::from_millis(500));
    
    // Clear the channel
    while let Ok(_) = rx.try_recv() {}
    
    // Modify both files again
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&meta_file)?;
        
        writeln!(file, "<!-- This modification should not be detected -->")?;
        file.flush()?;
    }
    
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&xml_file)?;
        
        writeln!(file, "<!-- This modification should be detected -->")?;
        file.flush()?;
    }
    
    // Wait for and collect events
    let after_unwatch_events = collect_events(&rx, Duration::from_secs(2));
    
    // Check that we only received events for the xml file
    assert!(!after_unwatch_events.iter().any(|e| e.path == meta_file.to_string_lossy()), 
           "Received events for meta file after unwatching");
    assert!(after_unwatch_events.iter().any(|e| e.path == xml_file.to_string_lossy()),
           "Did not receive events for xml file which is still being watched");
    
    // Keep temp_dir alive until the end of the test
    drop(temp_dir);
    
    Ok(())
}
