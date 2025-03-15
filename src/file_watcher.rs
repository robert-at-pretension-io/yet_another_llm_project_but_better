//! File watcher module for monitoring file system changes
//!
//! This module provides functionality to watch for file system events
//! and notify listeners when watched files are modified.

use notify::{Watcher as NotifyWatcher, RecursiveMode, Result as NotifyResult, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashSet;
use std::time::Duration;
use std::thread;

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
    pub path: PathBuf,
    /// Type of event that occurred
    pub event_type: FileEventType,
}

/// File watcher that monitors specified files for changes
pub struct FileWatcher {
    // Internal notify watcher
    _watcher: notify::RecommendedWatcher,
    // Set of files being watched
    watched_files: HashSet<PathBuf>,
    // Channel for receiving events from the watcher thread
    _event_thread: thread::JoinHandle<()>,
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
    /// A new FileWatcher instance or an error if the watcher couldn't be created
    pub fn new(sender: Sender<FileEvent>) -> NotifyResult<Self> {
        let (tx, rx) = channel();
        
        // Create the notify watcher
        let mut watcher = notify::recommended_watcher(tx)?;
        
        // Create a thread to process events
        let event_thread = thread::spawn(move || {
            Self::process_events(rx, sender);
        });
        
        Ok(FileWatcher {
            _watcher: watcher,
            watched_files: HashSet::new(),
            _event_thread: event_thread,
        })
    }
    
    /// Watch a specific file for changes
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to watch
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P) -> NotifyResult<()> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Add to our internal set of watched files
        self.watched_files.insert(path_buf.clone());
        
        // Start watching the file with notify
        // We use non-recursive mode since we're watching specific files
        self._watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
        
        Ok(())
    }
    
    /// Stop watching a specific file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to stop watching
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn unwatch_file<P: AsRef<Path>>(&mut self, path: P) -> NotifyResult<()> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Remove from our internal set
        self.watched_files.remove(&path_buf);
        
        // Stop watching the file with notify
        self._watcher.unwatch(path.as_ref())?;
        
        Ok(())
    }
    
    /// Check if a file is currently being watched
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check
    ///
    /// # Returns
    ///
    /// true if the file is being watched, false otherwise
    pub fn is_watching<P: AsRef<Path>>(&self, path: P) -> bool {
        self.watched_files.contains(&path.as_ref().to_path_buf())
    }
    
    /// Get a list of all files currently being watched
    ///
    /// # Returns
    ///
    /// A vector of paths being watched
    pub fn watched_files(&self) -> Vec<PathBuf> {
        self.watched_files.iter().cloned().collect()
    }
    
    // Process events from notify and convert them to our FileEvent type
    fn process_events(rx: Receiver<notify::Result<Event>>, sender: Sender<FileEvent>) {
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    // Convert notify events to our FileEvent type
                    if let Some(file_event) = Self::convert_event(event) {
                        // If sender is closed (receiver dropped), exit the thread
                        if sender.send(file_event).is_err() {
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {:?}", e);
                }
                Err(_) => {
                    // Channel closed, exit the thread
                    break;
                }
            }
        }
    }
    
    // Convert notify::Event to our FileEvent
    fn convert_event(event: Event) -> Option<FileEvent> {
        let event_type = match event.kind {
            EventKind::Create(_) => FileEventType::Created,
            EventKind::Modify(_) => FileEventType::Modified,
            EventKind::Remove(_) => FileEventType::Deleted,
            _ => return None, // Ignore other event types
        };
        
        // Get the path from the event
        // We only care about the first path in the event
        event.paths.first().map(|path| FileEvent {
            path: path.clone(),
            event_type,
        })
    }
}

// Implement Drop to ensure resources are cleaned up
impl Drop for FileWatcher {
    fn drop(&mut self) {
        // The watcher will be dropped automatically
        // The thread will exit when the channel is closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::sync::mpsc::channel;
    use tempfile::tempdir;
    
    #[test]
    fn test_file_watcher_creation() {
        let (tx, _rx) = channel();
        let watcher = FileWatcher::new(tx);
        assert!(watcher.is_ok());
    }
    
    #[test]
    fn test_watch_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        // Create the file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Initial content").unwrap();
        
        let (tx, _rx) = channel();
        let mut watcher = FileWatcher::new(tx).unwrap();
        
        // Watch the file
        assert!(watcher.watch_file(&file_path).is_ok());
        assert!(watcher.is_watching(&file_path));
        
        // Check watched files list
        let watched = watcher.watched_files();
        assert_eq!(watched.len(), 1);
        assert_eq!(watched[0], file_path);
    }
    
    #[test]
    fn test_unwatch_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        // Create the file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Initial content").unwrap();
        
        let (tx, _rx) = channel();
        let mut watcher = FileWatcher::new(tx).unwrap();
        
        // Watch and then unwatch the file
        watcher.watch_file(&file_path).unwrap();
        assert!(watcher.is_watching(&file_path));
        
        watcher.unwatch_file(&file_path).unwrap();
        assert!(!watcher.is_watching(&file_path));
        assert_eq!(watcher.watched_files().len(), 0);
    }
}
