//! File watcher module for monitoring file system changes
//!
//! This module provides functionality to watch for file system events
//! and notify listeners when watched files are modified.

use notify::{Watcher, DebouncedEvent, RecursiveMode};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashSet;
use std::time::Duration;
use std::thread;
use std::fs;

use crate::parser::{parse_document, Block};
use crate::executor::MetaLanguageExecutor;

/// Types of file events that can be detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEventType {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
}

/// Represents a file system event
#[derive(Debug, Clone)]
pub struct FileEvent {
    /// Path of the file that triggered the event
    pub path: String,
    /// Type of event that occurred
    pub event_type: FileEventType,
}

/// File watcher that monitors specified files for changes
pub struct FileWatcher {
    // Internal notify watcher
    watcher: notify::Watcher,
    // Set of files being watched
    watched_files: HashSet<PathBuf>,
    // Sender for file events
    sender: Sender<FileEvent>,
}

impl FileWatcher {
    /// Create a new file watcher that sends events to the provided sender
    ///
    /// # Arguments
    ///
    /// * `sender` - Channel to send file events when they occur
    ///
    /// # Returns
    ///
    /// A new FileWatcher instance
    pub fn new(sender: Sender<FileEvent>) -> Self {
        // Create an internal channel for the notify watcher
        let (tx, rx) = channel();
        
        // Create the notify watcher with a debounce time of 100ms
        let mut watcher = notify::Watcher::new(tx, Duration::from_millis(100))
            .expect("Failed to create file watcher");
        
        // Spawn a thread to handle events from the watcher
        let tx_clone = sender.clone();
        thread::spawn(move || {
            for event in rx {
                // Convert notify events to our FileEvent type
                if let Some(file_event) = Self::convert_event(event) {
                    // Send the event
                    if tx_clone.send(file_event).is_err() {
                        eprintln!("Error sending file event: receiver may have been dropped");
                        break;
                    }
                }
            }
        });
        
        FileWatcher {
            watcher,
            watched_files: HashSet::new(),
            sender,
        }
    }
    
    /// Watch a file or directory for changes
    ///
    /// # Arguments
    ///
    /// * `path` - Path to watch
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn watch(&mut self, path: String) -> Result<(), String> {
        let path_buf = PathBuf::from(&path);
        
        // Add to our internal set of watched files
        self.watched_files.insert(path_buf.clone());
        
        // Start watching the file with notify
        match self.watcher.watch(&path_buf, RecursiveMode::NonRecursive) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to watch path: {}", e)),
        }
    }
    
    // Convert notify::DebouncedEvent to our FileEvent
    fn convert_event(event: DebouncedEvent) -> Option<FileEvent> {
        let event_type = match event {
            DebouncedEvent::Create(_) => FileEventType::Created,
            DebouncedEvent::Write(_) => FileEventType::Modified,
            DebouncedEvent::Chmod(_) => FileEventType::Modified, // Treat chmod as modification
            DebouncedEvent::Remove(_) => FileEventType::Deleted,
            _ => return None, // Ignore other event types
        };
        
        // Get the path from the event
        match event {
            DebouncedEvent::Create(path) | 
            DebouncedEvent::Write(path) | 
            DebouncedEvent::Chmod(path) | 
            DebouncedEvent::Remove(path) => {
                path.to_str().map(|p| FileEvent {
                    path: p.to_string(),
                    event_type,
                })
            },
            _ => None,
        }
    }
}

// Implement Drop to ensure resources are cleaned up
impl Drop for FileWatcher {
    fn drop(&mut self) {
        // The watcher will be dropped automatically
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::sync::mpsc::channel;
    use tempfile::tempdir;
    use std::sync::{Arc, Mutex};
    
    #[test]
    fn test_file_watcher_creation() {
        let (tx, _rx) = channel();
        let watcher = FileWatcher::new(tx);
        // Just check that creation doesn't panic
    }
    
    #[test]
    fn test_watch_file_modification() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        // Create the file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Initial content").unwrap();
        
        let (tx, rx) = channel();
        let mut watcher = FileWatcher::new(tx);
        
        // Watch the file
        assert!(watcher.watch(file_path.to_str().unwrap().to_string()).is_ok());
        
        // Modify the file
        let mut file = OpenOptions::new().write(true).open(&file_path).unwrap();
        writeln!(file, "Modified content").unwrap();
        file.flush().unwrap();
        
        // Wait for the event
        let timeout = Duration::from_secs(5);
        let event = rx.recv_timeout(timeout);
        
        assert!(event.is_ok(), "Did not receive file event within timeout");
        let event = event.unwrap();
        assert_eq!(event.path, file_path.to_str().unwrap());
        assert_eq!(event.event_type, FileEventType::Modified);
    }
}
