//! File watcher module for monitoring file system changes
//!
//! This module provides functionality to watch for file system events
//! and notify listeners when watched files are modified.

use notify::{Watcher, DebouncedEvent, RecursiveMode, watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::collections::HashSet;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};

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
    watcher: notify::RecommendedWatcher,
    // Set of files being watched
    watched_files: HashSet<PathBuf>,
    // Flag to signal the background thread to stop
    running: Arc<Mutex<bool>>,
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
        let watcher = watcher(tx, Duration::from_millis(100))
            .expect("Failed to create file watcher");
        
        // Create a flag to signal when the watcher should stop
        let running = Arc::new(Mutex::new(true));
        let running_clone = running.clone();
        
        // Spawn a thread to handle events from the watcher
        thread::spawn(move || {
            while let Ok(true) = running_clone.lock().map(|guard| *guard) {
                // Try to receive an event with a timeout to allow checking the running flag
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(event) => {
                        // Convert notify events to our FileEvent type
                        if let Some(file_event) = Self::convert_event(event) {
                            // Send the event
                            if sender.send(file_event).is_err() {
                                eprintln!("Error sending file event: receiver may have been dropped");
                                break;
                            }
                        }
                    },
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Just a timeout, continue and check running flag
                        continue;
                    },
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        // Channel disconnected, exit the loop
                        eprintln!("File watcher channel disconnected");
                        break;
                    }
                }
            }
            eprintln!("File watcher thread shutting down");
        });
        
        FileWatcher {
            watcher,
            watched_files: HashSet::new(),
            running,
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
        
        // Check if the path exists
        if !path_buf.exists() {
            return Err(format!("Path does not exist: {}", path));
        }
        
        // Determine if this is a file or directory
        let is_dir = path_buf.is_dir();
        let mode = if is_dir {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        
        // Add to our internal set of watched files
        self.watched_files.insert(path_buf.clone());
        
        // Start watching the file with notify
        match self.watcher.watch(&path_buf, mode) {
            Ok(_) => {
                eprintln!("Now watching: {} ({})", path, if is_dir { "directory" } else { "file" });
                Ok(())
            },
            Err(e) => Err(format!("Failed to watch path: {}", e)),
        }
    }
    
    /// Stop watching a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to stop watching
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn unwatch(&mut self, path: &str) -> Result<(), String> {
        let path_buf = PathBuf::from(path);
        
        // Remove from our internal set
        self.watched_files.remove(&path_buf);
        
        // Stop watching with notify
        match self.watcher.unwatch(&path_buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to unwatch path: {}", e)),
        }
    }
    
    // Convert notify::DebouncedEvent to our FileEvent
    fn convert_event(event: DebouncedEvent) -> Option<FileEvent> {
        let (event_type, path) = match event {
            DebouncedEvent::Create(path) => (FileEventType::Created, path),
            DebouncedEvent::Write(path) => (FileEventType::Modified, path),
            DebouncedEvent::Chmod(path) => (FileEventType::Modified, path), // Treat chmod as modification
            DebouncedEvent::Remove(path) => (FileEventType::Deleted, path),
            DebouncedEvent::Rename(_, path) => (FileEventType::Modified, path), // Treat rename as modification of the new path
            _ => return None, // Ignore other event types
        };
        
        // Convert path to string and create FileEvent
        path.to_str().map(|p| FileEvent {
            path: p.to_string(),
            event_type,
        })
    }
}

// Implement Drop to ensure resources are cleaned up
impl Drop for FileWatcher {
    fn drop(&mut self) {
        eprintln!("Shutting down file watcher...");
        
        // Signal the background thread to stop
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
        
        // Unwatch all paths to clean up resources
        for path in self.watched_files.clone() {
            if let Err(e) = self.watcher.unwatch(&path) {
                eprintln!("Error unwatching path during shutdown: {}", e);
            } else if let Some(path_str) = path.to_str() {
                eprintln!("Stopped watching: {}", path_str);
            }
        }
        
        // Allow some time for the thread to clean up
        std::thread::sleep(Duration::from_millis(100));
        eprintln!("File watcher shutdown complete");
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
        let _watcher = FileWatcher::new(tx);
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
