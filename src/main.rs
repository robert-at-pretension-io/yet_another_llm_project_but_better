use std::env;
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::process;
use std::thread;
use std::time::Duration;

use anyhow::{Result, Context, anyhow};
use lazy_static::lazy_static;
use chrono::Local;
use ctrlc;

// Import from our library
use yet_another_llm_project_but_better::{
    parser::Block,
    file_watcher::{FileWatcher, FileEvent, FileEventType},
    executor::MetaLanguageExecutor
};

// Global configuration settings
lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::default()));
}

// Debug logging function that only prints if verbose mode is enabled
fn log_debug(message: &str) {
    if CONFIG.lock().unwrap().verbose {
        println!("[{}] DEBUG: {}", Local::now().format("%H:%M:%S"), message);
    }
}

// Configuration structure
struct Config {
    watch_mode: bool,
    watch_paths: Vec<String>,
    watch_extensions: Vec<String>,
    auto_execute: bool,
    answer_questions: bool,
    update_files: bool,
    verbose: bool,
    executor_map: HashMap<String, Arc<Mutex<MetaLanguageExecutor>>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watch_mode: true,
            watch_paths: vec![".".to_string()],
            watch_extensions: vec![".meta".to_string(), ".xml".to_string()],
            auto_execute: true,
            answer_questions: true,
            update_files: true,
            verbose: true,
            executor_map: HashMap::new(),
        }
    }
}

// Helper function to convert String errors to anyhow errors for file watcher
fn watch(watcher: &mut FileWatcher, path: String) -> Result<()> {
    watcher.watch(path.clone())
        .map_err(|e| anyhow!("Failed to watch path {}: {}", path, e))
}

// Process a file immediately
fn process_file(file_path: &str) -> Result<()> {
    log_debug(&format!("Starting process_file for: '{}'", file_path));
    let start_time = Instant::now();
    
    // Get detailed path information
    let path = Path::new(file_path);
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    
    // Print detailed path information for debugging
    log_debug(&format!("Path details:"));
    log_debug(&format!("  Original path: '{}'", file_path));
    log_debug(&format!("  Is absolute: {}", path.is_absolute()));
    log_debug(&format!("  Absolute path: '{}'", absolute_path.display()));
    log_debug(&format!("  Path exists (original): {}", path.exists()));
    log_debug(&format!("  Path exists (absolute): {}", absolute_path.exists()));
    
    // Check file extension
    if let Some(ext) = path.extension() {
        let ext_str = format!(".{}", ext.to_string_lossy());
        log_debug(&format!("File extension: '{}' for file: '{}'", ext_str, file_path));
    } else {
        log_debug(&format!("File '{}' has no extension", file_path));
    }
    
    let config = CONFIG.lock().unwrap();
    
    if config.verbose {
        println!("[{}] Processing file: {}", Local::now().format("%H:%M:%S"), file_path);
    }
    
    // Determine which path to use for reading
    let path_to_use = if path.exists() {
        log_debug("Using original path for file operations");
        file_path.to_string()
    } else if absolute_path.exists() {
        log_debug("Using absolute path for file operations");
        absolute_path.to_string_lossy().to_string()
    } else {
        log_debug(&format!("Neither original nor absolute path exists for '{}'", file_path));
        return Err(anyhow!("File not found: {}", file_path));
    };
    
    // Read the file
    log_debug(&format!("Reading file content from: '{}'", path_to_use));
    let content = match fs::read_to_string(&path_to_use) {
        Ok(content) => {
            log_debug(&format!("File read successfully, content length: {} bytes", content.len()));
            log_debug(&format!("First 100 chars of content: '{}'", 
                              if content.len() > 100 { &content[..100] } else { &content }));
            content
        },
        Err(e) => {
            log_debug(&format!("Failed to read file '{}': {}", path_to_use, e));
            return Err(anyhow!("Failed to read file: {}: {}", path_to_use, e));
        }
    };
    
    // Get or create an executor for this file
    // Use the original file_path as the key for the executor map
    // This ensures consistency when the same file is processed multiple times
    log_debug(&format!("Getting executor for file: '{}'", file_path));
    let executor = get_or_create_executor(file_path);
    let mut executor = executor.lock().unwrap();
    log_debug(&format!("Executor acquired successfully for '{}'", file_path));
    
    // Process the document
    log_debug(&format!("Beginning document processing for '{}'", file_path));
    println!("Processing document content: {} bytes", content.len());
    
    match executor.process_document(&content) {
        Ok(_) => {
            log_debug(&format!("Document '{}' processed successfully, found {} blocks", 
                              file_path, executor.blocks.len()));
            println!("Successfully parsed document, found {} blocks", executor.blocks.len());
            
            // Debug: Print all blocks found
            log_debug("Listing all blocks found:");
            for (name, block) in &executor.blocks {
                log_debug(&format!("Block: '{}' of type '{}'", name, block.block_type));
                println!("Found block: '{}' of type '{}'", name, block.block_type);
            }
        },
        Err(e) => {
            // Print the error but continue execution
            eprintln!("Error processing document {}: {}", file_path, e);
            log_debug(&format!("Error processing document '{}': {}", file_path, e));
        }
    }
    
    // Auto-execute blocks marked with auto_execute
    if config.auto_execute {
        log_debug("Auto-execute enabled, processing auto-execute blocks");
        auto_execute_blocks(&mut executor)?;
    } else {
        log_debug("Auto-execute disabled, skipping");
    }
    
    // Process question blocks that don't have responses
    if config.answer_questions {
        log_debug("Answer questions enabled, processing question blocks");
        answer_questions(&mut executor)?;
    } else {
        log_debug("Answer questions disabled, skipping");
    }
    
    // Update the file with results if configured
    if config.update_files {
        log_debug(&format!("Update files enabled, updating: {}", file_path));
        // Use the same path that was used for reading
        let path_to_use = if path.exists() {
            file_path.to_string()
        } else if absolute_path.exists() {
            absolute_path.to_string_lossy().to_string()
        } else {
            file_path.to_string() // Fallback to original path
        };
        log_debug(&format!("Using path for update: '{}'", path_to_use));
        update_file_with_results(&path_to_use, &mut executor)?;
    } else {
        log_debug("Update files disabled, skipping");
    }
    
    let elapsed = start_time.elapsed();
    log_debug(&format!("Completed process_file in {:.2?}", elapsed));
    
    if config.verbose {
        println!("[{}] Finished processing {} in {:.2?}",
                 Local::now().format("%H:%M:%S"), file_path, elapsed);
    }
    
    Ok(())
}

// Get or create an executor for a specific file
fn get_or_create_executor(file_path: &str) -> Arc<Mutex<MetaLanguageExecutor>> {
    log_debug(&format!("Entering get_or_create_executor for: {}", file_path));
    let mut config = CONFIG.lock().unwrap();
    
    if let Some(executor) = config.executor_map.get(file_path) {
        log_debug("Found existing executor, reusing");
        return executor.clone();
    }
    
    // Create a new executor
    log_debug("No existing executor found, creating new one");
    let executor = Arc::new(Mutex::new(MetaLanguageExecutor::new()));
    config.executor_map.insert(file_path.to_string(), executor.clone());
    log_debug(&format!("New executor created and stored for: {}", file_path));
    
    executor
}

// Execute blocks marked with auto_execute modifier
fn auto_execute_blocks(executor: &mut MetaLanguageExecutor) -> Result<()> {
    log_debug("Entering auto_execute_blocks");
    let mut executed_blocks = HashSet::new();
    
    // First collect the names of blocks to execute
    let blocks_to_execute: Vec<String> = executor.blocks.iter()
        .filter(|(_, block)| block.is_modifier_true("auto_execute"))
        .map(|(name, _)| name.clone())
        .collect();
    
    log_debug(&format!("Found {} blocks with auto_execute modifier", blocks_to_execute.len()));
    
    // Then execute each block
    for name in blocks_to_execute {
        log_debug(&format!("Executing auto_execute block: '{}'", name));
        if let Err(e) = executor.execute_block(&name) {
            eprintln!("Error executing block '{}': {}", name, e);
            log_debug(&format!("Error executing block '{}': {}", name, e));
        } else {
            executed_blocks.insert(name.clone());
            log_debug(&format!("Successfully executed block: '{}'", name));
        }
    }
    
    log_debug(&format!("Completed auto_execute_blocks, executed {} blocks", executed_blocks.len()));
    
    if !executed_blocks.is_empty() && CONFIG.lock().unwrap().verbose {
        println!("Auto-executed {} blocks", executed_blocks.len());
    }
    
    Ok(())
}

// Process question blocks without responses
fn answer_questions(executor: &mut MetaLanguageExecutor) -> Result<()> {
    log_debug("Entering answer_questions");
    let mut questions_answered = 0;
    
    // Identify question blocks
    let question_blocks: Vec<(String, Block)> = executor.blocks.iter()
        .filter(|(_, block)| block.block_type == "question")
        .filter(|(name, _)| {
            // Check if there's no response for this question
            let response_name = format!("{}_response", name);
            !executor.blocks.contains_key(&response_name)
        })
        .map(|(name, block)| (name.clone(), block.clone()))
        .collect();
    
    log_debug(&format!("Found {} question blocks without responses", question_blocks.len()));
    
    // Process each question block
    for (name, block) in question_blocks {
        log_debug(&format!("Processing question block: '{}', content length: {}", 
                          name, block.content.len()));
        
        if let Err(e) = executor.execute_block(&name) {
            eprintln!("Error executing question block '{}': {}", name, e);
            log_debug(&format!("Error executing question block '{}': {}", name, e));
        } else {
            questions_answered += 1;
            log_debug(&format!("Successfully processed question: '{}'", name));
        }
    }
    
    log_debug(&format!("Completed answer_questions, answered {} questions", questions_answered));
    
    if questions_answered > 0 && CONFIG.lock().unwrap().verbose {
        println!("Answered {} questions", questions_answered);
    }
    
    Ok(())
}

// Update the original file with execution results
fn update_file_with_results(file_path: &str, executor: &mut MetaLanguageExecutor) -> Result<()> {
    log_debug(&format!("Entering update_file_with_results for: {}", file_path));
    
    // Verify file exists before attempting to update
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(anyhow!("Cannot update file that doesn't exist: {}", file_path));
    }
    
    // Generate updated document content
    log_debug("Generating updated document content");
    let updated_content = executor.update_document()
        .with_context(|| format!("Failed to update document content for {}", file_path))?;
    log_debug(&format!("Generated updated content, length: {} bytes", updated_content.len()));
    
    // Read the current file content
    log_debug("Reading current file content for comparison");
    let current_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file for update: {}", file_path))?;
    log_debug(&format!("Current content length: {} bytes", current_content.len()));
    
    // Only write if content has changed
    if updated_content != current_content {
        log_debug("Content has changed, updating file");
        if CONFIG.lock().unwrap().verbose {
            println!("Updating file with execution results: {}", file_path);
        }
        
        fs::write(file_path, updated_content)
            .with_context(|| format!("Failed to write updated content to {}", file_path))?;
        log_debug("File updated successfully");
    } else {
        log_debug("Content unchanged, skipping file update");
    }
    
    log_debug("Completed update_file_with_results");
    Ok(())
}

// Handle a file system event from the watcher
fn handle_file_event(event: FileEvent) {
    log_debug(&format!("Received file event: {:?} for path: {}", event.event_type, event.path));
    let config = CONFIG.lock().unwrap();
    
    // Process file creations and modifications
    if event.event_type != FileEventType::Modified && event.event_type != FileEventType::Created {
        log_debug(&format!("Ignoring event type: {:?}", event.event_type));
        return;
    }
    
    // Check if the file exists (it might have been deleted right after being modified)
    let path = Path::new(&event.path);
    if !path.exists() {
        log_debug(&format!("File no longer exists: {}", event.path));
        return;
    }
    
    // Check if it's a file (not a directory)
    if !path.is_file() {
        log_debug(&format!("Path is not a file: {}", event.path));
        return;
    }
    
    // Check if the file extension is one we care about
    if let Some(ext) = path.extension() {
        let ext_str = format!(".{}", ext.to_string_lossy());
        log_debug(&format!("File extension: {}", ext_str));
        
        if !config.watch_extensions.contains(&ext_str) {
            log_debug(&format!("Ignoring file with unmonitored extension: {}", ext_str));
            return;
        }
    } else {
        log_debug("Ignoring file with no extension");
        return; // No extension, not a file we want to process
    }
    
    // Process the modified file
    log_debug(&format!("Processing modified file: {}", event.path));
    println!("[{}] File changed: {}", Local::now().format("%H:%M:%S"), event.path);
    
    // Small delay to ensure the file is fully written
    thread::sleep(Duration::from_millis(100));
    
    if let Err(e) = process_file(&event.path) {
        eprintln!("Error processing file {}: {}", event.path, e);
        log_debug(&format!("Error processing file {}: {}", event.path, e));
    } else {
        log_debug(&format!("Successfully processed modified file: {}", event.path));
    }
}

// Start the file watcher
fn start_file_watcher() -> Result<()> {
    log_debug("Entering start_file_watcher");
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new(tx);
    log_debug("FileWatcher created successfully");
    
    // Add watch paths
    let config = CONFIG.lock().unwrap();
    log_debug(&format!("Setting up {} watch paths", config.watch_paths.len()));
    
    // Track if we successfully set up at least one watch path
    let mut any_watch_success = false;
    
    for path in &config.watch_paths {
        log_debug(&format!("Attempting to watch path: {}", path));
        if let Err(e) = watch(&mut watcher, path.clone()) {
            eprintln!("Error watching path {}: {}", path, e);
            log_debug(&format!("Error watching path {}: {}", path, e));
        } else {
            any_watch_success = true;
            log_debug(&format!("Successfully watching path: {}", path));
            println!("[{}] Watching path: {}", Local::now().format("%H:%M:%S"), path);
            
            // Print which extensions we're monitoring
            println!("[{}] Monitoring files with extensions: {}", 
                     Local::now().format("%H:%M:%S"), 
                     config.watch_extensions.join(", "));
        }
    }
    
    // If we couldn't set up any watch paths, return an error
    if !any_watch_success {
        return Err(anyhow!("Failed to set up any watch paths"));
    }
    
    // Handle events in a loop
    println!("[{}] Watching for file changes. Press Ctrl+C to exit.", 
             Local::now().format("%H:%M:%S"));
    log_debug("Starting file watch event loop");
    drop(config); // Release the mutex lock
    
    // Set up Ctrl+C handler
    let (interrupt_tx, interrupt_rx) = channel();
    ctrlc::set_handler(move || {
        let _ = interrupt_tx.send(());
        println!("[{}] Received interrupt signal, shutting down...", 
                 Local::now().format("%H:%M:%S"));
    }).expect("Error setting Ctrl-C handler");
    
    // Main event loop
    'event_loop: loop {
        log_debug("Waiting for file events...");
        
        // Check for interrupt signal
        if interrupt_rx.try_recv().is_ok() {
            log_debug("Received interrupt signal, breaking event loop");
            break 'event_loop;
        }
        
        // Wait for file events with timeout to allow checking for interrupts
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                log_debug(&format!("Received file event for: {}", event.path));
                handle_file_event(event);
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Just a timeout, continue and check for interrupts
                continue;
            },
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("File watcher channel disconnected");
                log_debug("File watcher channel disconnected, exiting loop");
                break;
            }
        }
    }
    
    log_debug("Exiting start_file_watcher");
    Ok(())
}

// Parse command-line arguments and configure the application
fn parse_args() -> Result<Vec<String>> {
    log_debug("Entering parse_args");
    let args: Vec<String> = env::args().collect();
    log_debug(&format!("Processing {} command line arguments", args.len() - 1));
    
    // Debug: Print all arguments
    for (idx, arg) in args.iter().enumerate() {
        log_debug(&format!("Argument[{}]: '{}'", idx, arg));
    }
    
    let mut config = CONFIG.lock().unwrap();
    let mut files_to_process = Vec::new();
    
    let mut i = 1;
    while i < args.len() {
        log_debug(&format!("Processing argument[{}]: '{}'", i, args[i]));
        match args[i].as_str() {
            "--watch" | "-w" => {
                config.watch_mode = true;
                log_debug("Watch mode enabled");
            },
            "--path" | "-p" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    config.watch_paths = vec![args[i + 1].clone()];
                    log_debug(&format!("Watch path set to: '{}'", args[i + 1]));
                    i += 1;
                } else {
                    log_debug("Warning: --path specified without a value");
                }
            },
            "--extensions" | "-e" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    config.watch_extensions = args[i + 1]
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if s.starts_with('.') {
                                s.to_string()
                            } else {
                                format!(".{}", s)
                            }
                        })
                        .collect();
                    log_debug(&format!("Watch extensions set to: {:?}", config.watch_extensions));
                    i += 1;
                } else {
                    log_debug("Warning: --extensions specified without a value");
                }
            },
            "--no-auto-execute" => {
                config.auto_execute = false;
                log_debug("Auto-execute disabled");
            },
            "--no-questions" => {
                config.answer_questions = false;
                log_debug("Answer questions disabled");
            },
            "--no-update" => {
                config.update_files = false;
                log_debug("File updates disabled");
            },
            "--verbose" | "-v" => {
                config.verbose = true;
                log_debug("Verbose mode enabled");
            },
            "--help" | "-h" => {
                log_debug("Help requested, showing usage information");
                print_usage();
                process::exit(0);
            },
            // If not a known flag, assume it's a file to process
            _ if args[i].starts_with('-') => {
                log_debug(&format!("Unknown option: '{}'", args[i]));
                return Err(anyhow!("Unknown option: {}", args[i]));
            },
            // Individual file to process
            _ => {
                log_debug(&format!("Found file argument[{}]: '{}'", i, args[i]));
                log_debug(&format!("Adding '{}' to files_to_process", args[i]));
                files_to_process.push(args[i].clone());
            }
        }
        i += 1;
    }
    
    log_debug(&format!("Final files_to_process: {:?}", files_to_process));
    log_debug(&format!("Number of files to process: {}", files_to_process.len()));
    log_debug("Completed parse_args successfully");
    Ok(files_to_process)
}

// Print usage information
fn print_usage() {
    println!("META Programming Language Processor");
    println!("Usage: meta [options] [files...]");
    println!("");
    println!("Options:");
    println!("  -w, --watch         Watch for file changes and process automatically");
    println!("  -p, --path PATH     Path to watch for changes (default: current directory)");
    println!("  -e, --extensions    Comma-separated list of file extensions to watch (default: .meta,.xml)");
    println!("  --no-auto-execute   Don't auto-execute blocks marked with auto_execute");
    println!("  --no-questions      Don't process question blocks without responses");
    println!("  --no-update         Don't update files with execution results");
    println!("  -v, --verbose       Enable verbose output");
    println!("  -h, --help          Show this help message");
}

// Process a list of files
fn process_files(files: &[String]) -> Result<()> {
    log_debug(&format!("Entering process_files with {} files", files.len()));
    log_debug(&format!("Files to process: {:?}", files));
    
    if files.is_empty() {
        log_debug("No files specified to process, returning early");
        println!("No files specified to process");
        return Ok(());
    }
    
    for (index, file) in files.iter().enumerate() {
        log_debug(&format!("Processing file {}/{}: '{}'", index + 1, files.len(), file));
        log_debug(&format!("File path type check - absolute: {}, relative: {}", 
                          Path::new(file).is_absolute(), 
                          !Path::new(file).is_absolute()));
        println!("Processing file: {}", file);
        
        // Check if file exists
        let file_path = Path::new(file);
        let exists = file_path.exists();
        log_debug(&format!("File existence check: '{}' exists: {}", file, exists));
        
        if !exists {
            eprintln!("Error: File does not exist: {}", file);
            log_debug(&format!("File does not exist: '{}', skipping", file));
            continue;
        }
        
        // Log file metadata if possible
        if let Ok(metadata) = fs::metadata(file) {
            log_debug(&format!("File '{}' metadata - size: {} bytes, is_file: {}, is_dir: {}", 
                              file, metadata.len(), metadata.is_file(), metadata.is_dir()));
        } else {
            log_debug(&format!("Could not retrieve metadata for file: '{}'", file));
        }
        
        log_debug(&format!("About to call process_file for: '{}'", file));
        if let Err(e) = process_file(file) {
            eprintln!("Error processing file {}: {}", file, e);
            log_debug(&format!("Error processing file '{}': {}", file, e));
        } else {
            log_debug(&format!("Successfully processed file: '{}'", file));
            println!("Successfully processed file: {}", file);
        }
    }
    
    log_debug("Completed process_files");
    Ok(())
}

fn main() -> Result<()> {
    println!("[{}] Starting META Programming Language Processor", 
             Local::now().format("%H:%M:%S"));
    
    // Parse command-line arguments
    log_debug("Starting argument parsing");
    let files = match parse_args() {
        Ok(files) => {
            log_debug(&format!("parse_args returned {} files: {:?}", files.len(), files));
            files
        },
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            log_debug(&format!("Error parsing arguments: {}", e));
            print_usage();
            process::exit(1);
        }
    };
    
    log_debug(&format!("Found {} files to process: {:?}", files.len(), files));
    
    // Process specified files
    if !files.is_empty() {
        log_debug(&format!("Processing {} specified files: {:?}", files.len(), files));
        if let Err(e) = process_files(&files) {
            eprintln!("Error processing files: {}", e);
            log_debug(&format!("Error processing files: {}", e));
        } else {
            log_debug(&format!("Completed processing all {} specified files", files.len()));
        }
    } else {
        log_debug("No files to process from command line arguments");
    }
    
    // Start file watcher if in watch mode
    let config = CONFIG.lock().unwrap();
    if config.watch_mode {
        log_debug("Watch mode enabled, starting file watcher");
        drop(config); // Release the lock
        
        // If we're in watch mode, this will block until interrupted
        if let Err(e) = start_file_watcher() {
            eprintln!("Error in file watcher: {}", e);
            log_debug(&format!("Error in file watcher: {}", e));
            process::exit(1);
        }
    } else if files.is_empty() {
        // No files specified and not in watch mode, print usage
        log_debug("No files specified and not in watch mode, showing usage");
        print_usage();
    }
    
    log_debug("Program execution complete");
    Ok(())
}
