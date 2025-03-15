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
    FileEventType,
    executor::MetaLanguageExecutor
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
        "question" => {
            // For question blocks, we need to use the LLM client
            println!("Processing question: {}", block.content);
            
            // Create a new executor to handle the question
            let mut executor = MetaLanguageExecutor::new();
            
            // Add test_mode modifier to avoid actual API calls during testing
            let mut question_block = block.clone();
            question_block.add_modifier("test_mode", "true");
            
            match executor.execute_question(&question_block, &block.content) {
                Ok(response) => Ok(response),
                Err(e) => Err(format!("Failed to execute question: {}", e))
            }
        },
        _ => Err(format!("Block type '{}' is not executable", block.block_type))
    }
}

/// Checks if a block should be auto-executed
fn should_auto_execute(block: &Block) -> bool {
    println!("DEBUG: Checking auto-execute for block: {:?}", block.name);
    println!("DEBUG: Block type: '{}'", block.block_type);
    println!("DEBUG: Block modifiers: {:?}", block.modifiers);
    
    // Check for auto_execute modifier
    let auto_execute = block.is_modifier_true("auto_execute");
    println!("DEBUG: auto_execute modifier is {}", auto_execute);
    
    if auto_execute {
        return true;
    }
    
    // Check for auto-executable block types
    let is_auto_executable_type = matches!(block.block_type.as_str(), 
             "shell" | "code:python" | "code:javascript" | "code:ruby" | "question");
    
    println!("DEBUG: is auto-executable type: {}", is_auto_executable_type);
    
    is_auto_executable_type
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
    
    // Create an executor for more advanced blocks like questions
    let mut executor = MetaLanguageExecutor::new();
    executor.process_document(&content)
        .map_err(|e| anyhow!("Executor error: {}", e))?;
    
    // Print detailed information about each block
    for (i, block) in blocks.iter().enumerate() {
        println!("\nDEBUG: Block #{} details:", i + 1);
        println!("  Type: '{}'", block.block_type);
        println!("  Name: {:?}", block.name);
        println!("  Modifiers: {:?}", block.modifiers);
        println!("  Content length: {} bytes", block.content.len());
        println!("  Content preview: '{}'", 
                 block.content.chars().take(50).collect::<String>().replace("\n", "\\n"));
        println!("  Children count: {}", block.children.len());
    }
    
    // Execute auto-executable blocks
    for (i, block) in blocks.iter().enumerate() {
        println!("\nDEBUG: Evaluating block #{} for execution:", i + 1);
        let should_execute = should_auto_execute(block);
        println!("DEBUG: should_auto_execute result: {}", should_execute);
        
        if should_execute {
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

