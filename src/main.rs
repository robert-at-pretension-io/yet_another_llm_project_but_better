use std::env;
use std::fs;
use std::path::Path;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::{Duration, Instant};
use std::process;

use anyhow::{Result, Context, anyhow};
use chrono::Local;
use ctrlc;

// Import from our library
use yet_another_llm_project_but_better::{
    parser::Block,
    file_watcher::{FileWatcher, FileEvent, FileEventType},
    executor::MetaLanguageExecutor
};

// Simple configuration structure
struct Config {
    watch_mode: bool,
    verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watch_mode: false,
            verbose: true,
        }
    }
}

// Debug logging function
fn debug(message: &str) {
    println!("[{}] DEBUG: {}", Local::now().format("%H:%M:%S"), message);
}

// Process a file
fn process_file(file_path: &str) -> Result<()> {
    let start_time = Instant::now();
    
    // Get path information
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(anyhow!("File not found: {}", file_path));
    }
    
    // Check if this is an XML file
    let is_xml_file = path.extension()
        .map(|ext| ext.to_string_lossy().to_lowercase() == "xml")
        .unwrap_or(false);
    
    if is_xml_file {
        println!("Processing XML file: {}", file_path);
    } else {
        println!("Processing file: {}", file_path);
    }
    
    // Read the file
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;
    
    println!("File content length: {} bytes", content.len());
    println!("Content preview: '{}'", 
             if content.len() > 50 { &content[..50] } else { &content });
    
    // Create an executor for this file
    let mut executor = MetaLanguageExecutor::new();
    println!("Created executor: {}", executor.instance_id);
    
    // Process the document
    match executor.process_document(&content) {
        Ok(_) => {
            println!("Successfully parsed document, found {} blocks", executor.blocks.len());
            
            // Print all blocks found
            for (name, block) in &executor.blocks {
                println!("Found block: '{}' of type '{}'", name, block.block_type);
            }
        },
        Err(e) => {
            eprintln!("Error processing document {}: {}", file_path, e);
            return Err(anyhow!("Failed to process document: {}", e));
        }
    }
    
    // Execute blocks marked with auto_execute
    let blocks_to_execute: Vec<String> = executor.blocks.iter()
        .filter(|(_, block)| block.is_modifier_true("auto_execute"))
        .map(|(name, _)| name.clone())
        .collect();
    
    if !blocks_to_execute.is_empty() {
        println!("Found {} blocks with auto_execute modifier", blocks_to_execute.len());
        
        for name in blocks_to_execute {
            println!("Executing auto_execute block: '{}'", name);
            if let Err(e) = executor.execute_block(&name) {
                eprintln!("Error executing block '{}': {}", name, e);
            } else {
                println!("Successfully executed block: '{}'", name);
            }
        }
    }
    
    // Process question blocks without responses
    let question_blocks: Vec<(String, Block)> = executor.blocks.iter()
        .filter(|(_, block)| block.block_type == "question")
        .filter(|(name, _)| {
            let response_name = format!("{}_response", name);
            !executor.blocks.contains_key(&response_name)
        })
        .map(|(name, block)| (name.clone(), block.clone()))
        .collect();
    
    if !question_blocks.is_empty() {
        println!("Found {} question blocks without responses", question_blocks.len());
        
        for (name, _) in &question_blocks {
            println!("Processing question: '{}'", name);
            if let Err(e) = executor.execute_block(name) {
                eprintln!("Error executing question block '{}': {}", name, e);
            } else {
                println!("Successfully processed question: '{}'", name);
            }
        }
    }
    
    // Update the file with results
    let updated_content = executor.update_document()
        .with_context(|| format!("Failed to update document content for {}", file_path))?;
    
    // Only write if content has changed
    if updated_content != content {
        println!("Updating file with execution results: {}", file_path);
        fs::write(file_path, updated_content)
            .with_context(|| format!("Failed to write updated content to {}", file_path))?;
    } else {
        println!("Content unchanged, no update needed");
    }
    
    let elapsed = start_time.elapsed();
    println!("Finished processing {} in {:.2?}", file_path, elapsed);
    
    Ok(())
}

// Start the file watcher
fn start_file_watcher() -> Result<()> {
    println!("Starting file watcher...");
    
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new(tx);
    
    // Watch current directory for XML files
    let watch_path = ".".to_string();
    watcher.watch(watch_path.clone())
        .map_err(|e| anyhow!("Failed to watch path {}: {}", watch_path, e))?;
    
    println!("Watching directory: {}", watch_path);
    println!("Monitoring files with extension: .xml");
    
    // Set up Ctrl+C handler
    let (interrupt_tx, interrupt_rx) = channel();
    ctrlc::set_handler(move || {
        let _ = interrupt_tx.send(());
        println!("Received interrupt signal, shutting down...");
    }).expect("Error setting Ctrl-C handler");
    
    println!("Watching for file changes. Press Ctrl+C to exit.");
    
    // Main event loop
    loop {
        // Check for interrupt signal
        if interrupt_rx.try_recv().is_ok() {
            break;
        }
        
        // Wait for file events with timeout to allow checking for interrupts
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                // Only process XML files
                if event.path.to_lowercase().ends_with(".xml") &&
                   (event.event_type == FileEventType::Modified || 
                    event.event_type == FileEventType::Created) {
                    
                    let path = Path::new(&event.path);
                    if path.exists() && path.is_file() {
                        println!("File changed: {}", event.path);
                        
                        // Small delay to ensure the file is fully written
                        std::thread::sleep(Duration::from_millis(100));
                        
                        if let Err(e) = process_file(&event.path) {
                            eprintln!("Error processing file {}: {}", event.path, e);
                        }
                    }
                }
            },
            Err(RecvTimeoutError::Timeout) => {
                // Just a timeout, continue and check for interrupts
                continue;
            },
            Err(RecvTimeoutError::Disconnected) => {
                eprintln!("File watcher channel disconnected");
                break;
            }
        }
    }
    
    println!("File watcher stopped");
    Ok(())
}

fn main() -> Result<()> {
    println!("META XML Processor");
    
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let config = Config::default();
    
    // Process file if provided
    if args.len() > 1 {
        let file_path = &args[1];
        println!("Processing file: {}", file_path);
        
        if let Err(e) = process_file(file_path) {
            eprintln!("Error processing file {}: {}", file_path, e);
            process::exit(1);
        }
    } else if config.watch_mode {
        // Start file watcher if no file specified and watch mode is enabled
        if let Err(e) = start_file_watcher() {
            eprintln!("Error in file watcher: {}", e);
            process::exit(1);
        }
    } else {
        // No file specified and not in watch mode, print usage
        println!("Usage: meta [file.xml]");
        println!("       meta --watch");
    }
    
    Ok(())
}
