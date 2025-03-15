use std::env;
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::fs;

use yet_another_llm_project_but_better::file_watcher::{FileEvent, FileEventType, FileWatcher};
use yet_another_llm_project_but_better::parser::{parse_document, Block};
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;

fn main() {
    // Get file path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    }
    
    let file_path = &args[1];
    println!("Watching file: {}", file_path);
    
    // Create a channel for file events
    let (tx, rx) = channel();
    
    // Create a file watcher
    let mut watcher = FileWatcher::new(tx);
    
    // Start watching the specified file
    match watcher.watch(file_path.clone()) {
        Ok(_) => println!("Started watching file successfully"),
        Err(e) => {
            eprintln!("Error watching file: {}", e);
            process::exit(1);
        }
    }
    
    // Create an executor for running blocks
    let mut executor = MetaLanguageExecutor::new();
    
    // Process file events
    println!("\nWaiting for file changes... (Press Ctrl+C to exit)");
    println!("Watching file: {}", file_path);
    
    loop {
        // Check for file events with a timeout
        if let Ok(event) = rx.recv_timeout(Duration::from_secs(1)) {
            match event.event_type {
                FileEventType::Created | FileEventType::Modified => {
                    println!("\nFile changed: {}", event.path);
                    
                    // Read the file content
                    match fs::read_to_string(&event.path) {
                        Ok(content) => {
                            // Parse the document
                            match parse_document(&content) {
                                Ok(blocks) => {
                                    println!("Found {} blocks in the document", blocks.len());
                                    
                                    // Process each block
                                    for block in blocks {
                                        process_block(&mut executor, &block);
                                    }
                                },
                                Err(e) => eprintln!("Error parsing document: {}", e),
                            }
                        },
                        Err(e) => eprintln!("Error reading file: {}", e),
                    }
                },
                FileEventType::Deleted => {
                    println!("File deleted: {}", event.path);
                },
            }
        }
        
        // Small delay to prevent CPU spinning
        thread::sleep(Duration::from_millis(100));
    }
}

fn process_block(executor: &mut MetaLanguageExecutor, block: &Block) {
    // Skip blocks that aren't executable
    if !executor.is_executable_block(block) {
        println!("Skipping non-executable block: {} (type: {})", 
                 block.name.as_deref().unwrap_or("unnamed"), 
                 block.block_type);
        return;
    }
    
    // Get block name or generate a temporary one
    let block_name = match &block.name {
        Some(name) => name.clone(),
        None => format!("unnamed_block_{}", block.block_type)
    };
    
    println!("\n==================================================");
    println!("Executing block: {} (type: {})", block_name, block.block_type);
    println!("==================================================");
    
    // Register the block with the executor
    executor.blocks.insert(block_name.clone(), block.clone());
    
    // Execute the block
    match executor.execute_block(&block_name) {
        Ok(output) => {
            println!("\nExecution successful!");
            println!("Output:\n{}", output);
        },
        Err(e) => {
            eprintln!("\nExecution failed: {}", e);
        }
    }
}
