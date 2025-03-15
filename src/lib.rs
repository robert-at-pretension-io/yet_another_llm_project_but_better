//! Meta Programming Language for AI document augmentation
//! 
//! This library provides parsing and execution capabilities 
//! for the Meta Programming Language, which allows documents
//! to contain executable blocks of code, data definitions,
//! and AI-enhanced content.

pub mod parser;
pub mod executor;
pub mod file_watcher;

// Re-export common types
pub use parser::{Block, parse_document};
pub use executor::MetaLanguageExecutor;
pub use file_watcher::{FileWatcher, FileEvent, FileEventType};
