use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use anyhow::{Context, anyhow};

// Import from our library
use yet_another_llm_project_but_better::{
    Block, 
    parse_document, 
    FileWatcher, 
    FileEvent, 
    FileEventType
};

/// Executes a block based on its type
fn execute_block(block: &Block) -> Result<String, String> {
    match block.block_type.as_str() {
        "code:python" => {
            // Try to execute Python code with "python" first
            let output = Command::new("python")
                .arg("-c")
                .arg(&block.content)
                .output();
            
            match output {
                Ok(output) => {
                    if output.status.success() {
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    } else {
                        Err(String::from_utf8_lossy(&output.stderr).to_string())
                    }
                },
                Err(e) => {
                    // If "python" fails, try "python3"
                    println!("Python command failed: {}. Trying python3...", e);
                    let output = Command::new("python3")
                        .arg("-c")
                        .arg(&block.content)
                        .output()
                        .map_err(|e| format!("Failed to execute Python3: {}", e))?;
                    
                    if output.status.success() {
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    } else {
                        Err(String::from_utf8_lossy(&output.stderr).to_string())
                    }
                }
            }
        },
        "shell" => {
            // Execute shell command
            let output = Command::new("sh")
                .arg("-c")
                .arg(&block.content)
                .output()
                .map_err(|e| format!("Failed to execute shell command: {}", e))?;
            
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        _ => Err(format!("Block type '{}' is not executable", block.block_type))
    }
}

/// Checks if a block should be auto-executed
fn should_auto_execute(block: &Block) -> bool {
    // Check for auto_execute modifier
    if block.is_modifier_true("auto_execute") {
        return true;
    }
    
    // Check for auto-executable block types
    matches!(block.block_type.as_str(), 
             "shell" | "code:python" | "code:javascript" | "code:ruby")
}

/// Process a file, parsing and executing blocks as needed
fn process_file(file_path: &Path) -> Result<(), anyhow::Error> {
    println!("Processing file: {}", file_path.display());
    
    // Read file content
    let content = fs::read_to_string(file_path)
        .context("Failed to read file")?;
    
    // Parse document to find blocks
    let blocks = parse_document(&content)
        .map_err(|e| anyhow!("Parser error: {}", e))?;
    
    println!("Found {} blocks in file", blocks.len());
    
    // Execute auto-executable blocks
    for block in &blocks {
        if should_auto_execute(block) {
            println!("Auto-executing block: {}{}", 
                     block.block_type, 
                     block.name.as_ref().map_or(String::new(), |n| format!(" ({})", n)));
            
            match execute_block(block) {
                Ok(output) => {
                    println!("=== Output from block {} ===", 
                             block.name.as_ref().unwrap_or(&block.block_type));
                    println!("{}", output);
                    println!("=== End of output ===");
                },
                Err(e) => {
                    eprintln!("Error executing block: {}", e);
                    println!("=== Error in block {} ===", 
                             block.name.as_ref().unwrap_or(&block.block_type));
                    println!("{}", e);
                    println!("=== End of error ===");
                }
            }
        }
    }
    
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    // Get file path from command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check for --no-watch flag
    let no_watch = args.contains(&String::from("--no-watch"));
    
    // Find the file path (skip program name and any flags)
    let mut file_path = None;
    for arg in &args[1..] {
        if !arg.starts_with("--") {
            file_path = Some(PathBuf::from(arg));
            break;
        }
    }
    
    // Ensure we have a file path
    let file_path = match file_path {
        Some(path) => path,
        None => {
            eprintln!("Usage: {} [--no-watch] <file_path>", args[0]);
            std::process::exit(1);
        }
    };
    
    if !file_path.exists() {
        eprintln!("File does not exist: {}", file_path.display());
        std::process::exit(1);
    }
    
    // Initial processing of the file
    process_file(&file_path)?;
    
    // Exit early if --no-watch flag is present
    if no_watch {
        println!("Processed file once, exiting (--no-watch flag detected)");
        return Ok(());
    }
    
    // Set up file watcher using our library's FileWatcher
    println!("Watching file for changes: {}", file_path.display());
    
    let (tx, rx) = channel();
    let mut file_watcher = FileWatcher::new(tx);
    
    // Start watching the file
    file_watcher.watch(file_path.to_string_lossy().to_string())
        .map_err(|e| anyhow!("Failed to watch file: {}", e))?;
    
    // Track last modification time to avoid duplicate events
    let mut last_modified = Instant::now();
    
    // Watch for file changes
    loop {
        match rx.recv() {
            Ok(FileEvent { path, event_type: FileEventType::Modified }) => {
                // Avoid processing the same change multiple times
                let now = Instant::now();
                if now.duration_since(last_modified) < Duration::from_millis(100) {
                    continue;
                }
                last_modified = now;
                
                println!("File changed: {}", path);
                if let Err(e) = process_file(&PathBuf::from(&path)) {
                    eprintln!("Error processing file after change: {:?}", e);
                }
            },
            Ok(FileEvent { event_type: FileEventType::Created, .. }) => {
                println!("File was created, but we're only watching for modifications");
            },
            Ok(FileEvent { event_type: FileEventType::Deleted, .. }) => {
                println!("File was deleted, exiting watch loop");
                break;
            },
            Err(e) => {
                eprintln!("Watch channel error: {:?}", e);
                break;
            },
        }
    }
    
    Ok(())
}

