use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Context, anyhow};
use lazy_static::lazy_static;

// Import from our library
use yet_another_llm_project_but_better::{
    Block, 
    parse_document, 
    FileWatcher, 
    FileEvent, 
    FileEventType,
    executor::MetaLanguageExecutor
};

// Global map to store executor instances by file path
lazy_static! {
    static ref EXECUTORS: Mutex<HashMap<String, Arc<Mutex<MetaLanguageExecutor>>>> = Mutex::new(HashMap::new());
}

/// Executes a block based on its type
/// Returns a tuple with (execution_result, file_needs_update)
fn execute_block(block: &Block, file_path: &Path) -> Result<(String, bool), String> {
    // Get the file path as a string for the executor map
    let file_path_str = file_path.to_string_lossy().to_string();
    
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
                        Ok((String::from_utf8_lossy(&output.stdout).to_string(), false))
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
                        Ok((String::from_utf8_lossy(&output.stdout).to_string(), false))
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
                Ok((String::from_utf8_lossy(&output.stdout).to_string(), false))
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        "question" => {
            // For question blocks, we need to use the LLM client
            println!("Processing question: {}", block.content);
            
            // Check if the file already has a response block for this specific question
            let file_content = fs::read_to_string(file_path).unwrap_or_default();
            
            // Get the question block name or use a default identifier
            let block_identifier = block.name.as_ref()
                .map(|name| format!("for=\"{}\"", name))
                .unwrap_or_default();
                
            // Check for a complete response block that matches this question
            let has_response = if !block_identifier.is_empty() {
                // Look for a response block with the specific identifier
                file_content.contains(&format!("[response {}]", block_identifier)) && 
                file_content.contains("[/response]")
            } else {
                // For unnamed blocks, check if there's any response after this question
                if let Some(question_end_pos) = file_content.find("[/question]") {
                    let after_question = &file_content[question_end_pos..];
                    after_question.contains("[response") && after_question.contains("[/response]")
                } else {
                    false
                }
            };
            
            if has_response {
                println!("File already has a response block for this question, skipping execution");
                return Ok(("Response already exists in file".to_string(), false));
            }
            
            // Get or create an executor for this file
            let executor_arc = {
                let mut executors = EXECUTORS.lock().unwrap();
                executors.entry(file_path_str.clone())
                    .or_insert_with(|| {
                        let mut executor = MetaLanguageExecutor::new();
                        // Initialize with current file content
                        executor.process_document(&file_content).unwrap_or_default();
                        Arc::new(Mutex::new(executor))
                    })
                    .clone()
            };
            
            // Get a mutable reference to the executor
            let mut executor = executor_arc.lock().unwrap();
            
            // Add test_mode modifier to avoid actual API calls during testing
            let mut question_block = block.clone();
            // question_block.add_modifier("test_mode", "true");
            
            match executor.execute_question(&question_block, &block.content) {
                Ok(response) => {
                    println!("DEBUG: Question execution successful, response length: {} bytes", response.len());
                    println!("DEBUG: Response preview: {}", response.chars().take(100).collect::<String>());
                    
                    // Store the response in the executor
                    if let Some(name) = &block.name {
                        let response_name = format!("{}_response", name);
                        println!("DEBUG: Storing response with key: '{}'", response_name);
                        executor.outputs.insert(response_name, response.clone());
                    } else {
                        // For unnamed blocks, use a generic key
                        println!("DEBUG: Storing response with generic key 'question_response'");
                        executor.outputs.insert("question_response".to_string(), response.clone());
                    }
                    
                    // Create a response block with proper attribution to the question
                    // Find the end of the question block
                    if let Some(pos) = file_content.find("[/question]") {
                        println!("DEBUG: Found [/question] at position {}", pos);
                        
                        // Create response block with attribution if the question has a name
                        let response_block = if !block_identifier.is_empty() {
                            format!("\n\n[response {}]\n{}\n[/response]", block_identifier, response)
                        } else {
                            format!("\n\n[response]\n{}\n[/response]", response)
                        };
                        
                        println!("DEBUG: Created response block with length: {} bytes", response_block.len());
                        println!("DEBUG: Response block preview: {}", 
                                 response_block.chars().take(100).collect::<String>().replace("\n", "\\n"));
                        
                        // File needs to be updated with the response
                        let mut needs_update = true;
                    } else {
                        println!("DEBUG: Could not find [/question] tag in the document");
                    }
                    
                    Ok((response, needs_update))
                },
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
    
    // Get the file path as a string for the executor map
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Read file content
    let content = fs::read_to_string(file_path)
        .context("Failed to read file")?;
    
    // Parse document to find blocks
    let blocks = parse_document(&content)
        .map_err(|e| anyhow!("Parser error: {}", e))?;
    
    println!("Found {} blocks in file", blocks.len());
    
    // Get or create an executor for this file
    let executor_arc = {
        let mut executors = EXECUTORS.lock().unwrap();
        executors.entry(file_path_str.clone())
            .or_insert_with(|| {
                let mut executor = MetaLanguageExecutor::new();
                // Initialize with current file content
                executor.process_document(&content).unwrap_or_default();
                Arc::new(Mutex::new(executor))
            })
            .clone()
    };
    
    // Get a mutable reference to the executor
    let mut executor = executor_arc.lock().unwrap();
    
    // Update the executor with the current document content
    executor.process_document(&content)
        .map_err(|e| anyhow!("Executor error: {}", e))?;
    
    // Track if we need to update the file
    let mut file_updated = false;
    
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
            
            match execute_block(block, file_path) {
                Ok((output, needs_update)) => {
                    println!("=== Output from block {} ===", 
                             block.name.as_ref().unwrap_or(&block.block_type));
                    println!("{}", output);
                    println!("=== End of output ===");
                    
                    // Update the file_updated flag based on the execution result
                    if needs_update {
                        file_updated = true;
                    }
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
    
    // If we executed any blocks that require file updates, update the file using the executor
    if file_updated {
        println!("DEBUG: File update required, calling executor.update_document()");
        
        // Get the updated document content from the executor
        let updated_content = match executor.update_document() {
            Ok(content) => {
                println!("DEBUG: executor.update_document() succeeded, content length: {} bytes", content.len());
                println!("DEBUG: Updated content preview: {}", 
                         content.chars().take(100).collect::<String>().replace("\n", "\\n"));
                content
            },
            Err(e) => {
                println!("DEBUG: executor.update_document() failed: {}", e);
                return Err(anyhow!("Failed to update document: {}", e));
            }
        };
        
        // Check if the content actually changed
        let current_content = match fs::read_to_string(file_path) {
            Ok(content) => {
                println!("DEBUG: Read current file content, length: {} bytes", content.len());
                content
            },
            Err(e) => {
                println!("DEBUG: Failed to read current file content: {}", e);
                return Err(anyhow!("Failed to read current file: {}", e));
            }
        };
        
        if current_content != updated_content {
            println!("DEBUG: Content has changed, writing updated content to file via executor.update_document()");
            
            // Write the updated content back to the file
            match fs::write(file_path, &updated_content) {
                Ok(_) => println!("DEBUG: Successfully wrote updated content to file"),
                Err(e) => {
                    println!("DEBUG: Failed to write updated file: {}", e);
                    return Err(anyhow!("Failed to write updated file: {}", e));
                }
            }
            
            println!("Updated file with execution results");
        } else {
            println!("DEBUG: Content unchanged, no file update needed");
            println!("No changes needed to file");
        }
    } else {
        println!("DEBUG: No file update required");
    }
    
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    // Initialize the EXECUTORS lazy_static
    {
        let _executors = EXECUTORS.lock().unwrap();
        println!("Initialized EXECUTORS static");
    }
    
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
    
    // Track if we're currently in a cooldown period after processing
    let mut in_cooldown = false;
    
    // Debounce buffer for file events
    let mut pending_event = false;
    let mut last_event_path = String::new();
    
    // Watch for file changes
    loop {
        // If we have a pending event and we're not in cooldown, process it
        if pending_event && !in_cooldown {
            let path = last_event_path.clone();
            pending_event = false;
            
            // Read the current content to check if we need to process it
            match fs::read_to_string(&path) {
                Ok(current_content) => {
                    println!("DEBUG: Read file after change, content length: {} bytes", current_content.len());
                },
                Err(e) => {
                    println!("DEBUG: Failed to read file after change: {}", e);
                    continue;
                }
            }
            
            // Process the file
            println!("File changed: {}", path);
            println!("DEBUG: Processing file after change");
            
            // Enter cooldown before processing to prevent reacting to our own changes
            in_cooldown = true;
            println!("DEBUG: Entering processing cooldown");
            
            if let Err(e) = process_file(&PathBuf::from(&path)) {
                eprintln!("Error processing file after change: {:?}", e);
            } else {
                println!("DEBUG: Successfully processed file after change");
            }
            
            // After processing, wait for a cooldown period before accepting new events
            // This prevents reacting to our own file modifications
            std::thread::sleep(Duration::from_millis(1000));
            in_cooldown = false;
            println!("DEBUG: Exiting processing cooldown");
            
            // Reset the last_modified time after processing
            last_modified = Instant::now();
            continue;
        }
        
        // Use a timeout to check for pending events periodically
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(FileEvent { path, event_type: FileEventType::Modified }) => {
                // Skip events during cooldown
                if in_cooldown {
                    println!("DEBUG: Ignoring file change during cooldown period");
                    continue;
                }
                
                // Avoid processing the same change multiple times with debouncing
                let now = Instant::now();
                if now.duration_since(last_modified) < Duration::from_millis(500) {
                    // Update the pending event but don't process yet
                    pending_event = true;
                    last_event_path = path.clone();
                    println!("DEBUG: Debouncing file change event");
                    continue;
                }
                
                // Mark this event as pending
                pending_event = true;
                last_event_path = path.clone();
                last_modified = now;
                
                // Wait for debounce period before processing
                println!("DEBUG: Waiting for debounce period (500ms)");
                std::thread::sleep(Duration::from_millis(500));
            },
            Ok(FileEvent { event_type: FileEventType::Created, .. }) => {
                println!("File was created, but we're only watching for modifications");
            },
            Ok(FileEvent { event_type: FileEventType::Deleted, .. }) => {
                println!("File was deleted, exiting watch loop");
                break;
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // This is just a timeout for our polling loop, not an error
                continue;
            },
            Err(e) => {
                eprintln!("Watch channel error: {:?}", e);
                break;
            },
        }
    }
    
    Ok(())
}

