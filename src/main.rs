use std::env;
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::process;

use anyhow::{Result, Context, anyhow};
use lazy_static::lazy_static;
use chrono::Local;

// Import from our library
use yet_another_llm_project_but_better::{
    parser::{parse_document, Block},
    file_watcher::{FileWatcher, FileEvent, FileEventType},
    executor::MetaLanguageExecutor
};

// Global configuration settings
lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::default()));
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
            watch_mode: false,
            watch_paths: vec![".".to_string()],
            watch_extensions: vec![".meta".to_string(), ".xml".to_string()],
            auto_execute: true,
            answer_questions: true,
            update_files: true,
            verbose: false,
            executor_map: HashMap::new(),
        }
    }
}

// Process a file immediately
fn process_file(file_path: &str) -> Result<()> {
    let start_time = Instant::now();
    let config = CONFIG.lock().unwrap();
    
    if config.verbose {
        println!("[{}] Processing file: {}", Local::now().format("%H:%M:%S"), file_path);
    }
    
    // Read the file
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;
    
    // Get or create an executor for this file
    let executor = get_or_create_executor(file_path);
    let mut executor = executor.lock().unwrap();
    
    // Process the document
    executor.process_document(&content)
        .with_context(|| format!("Failed to process document: {}", file_path))?;
    
    // Auto-execute blocks marked with auto_execute
    if config.auto_execute {
        auto_execute_blocks(&mut executor)?;
    }
    
    // Process question blocks that don't have responses
    if config.answer_questions {
        answer_questions(&mut executor)?;
    }
    
    // Update the file with results if configured
    if config.update_files {
        update_file_with_results(file_path, &mut executor)?;
    }
    
    if config.verbose {
        let elapsed = start_time.elapsed();
        println!("[{}] Finished processing {} in {:.2?}", 
                 Local::now().format("%H:%M:%S"), file_path, elapsed);
    }
    
    Ok(())
}

// Get or create an executor for a specific file
fn get_or_create_executor(file_path: &str) -> Arc<Mutex<MetaLanguageExecutor>> {
    let mut config = CONFIG.lock().unwrap();
    
    if let Some(executor) = config.executor_map.get(file_path) {
        return executor.clone();
    }
    
    // Create a new executor
    let executor = Arc::new(Mutex::new(MetaLanguageExecutor::new()));
    config.executor_map.insert(file_path.to_string(), executor.clone());
    
    executor
}

// Execute blocks marked with auto_execute modifier
fn auto_execute_blocks(executor: &mut MetaLanguageExecutor) -> Result<()> {
    let mut executed_blocks = HashSet::new();
    
    // First collect the names of blocks to execute
    let blocks_to_execute: Vec<String> = executor.blocks.iter()
        .filter(|(_, block)| block.is_modifier_true("auto_execute"))
        .map(|(name, _)| name.clone())
        .collect();
    
    // Then execute each block
    for name in blocks_to_execute {
        if let Err(e) = executor.execute_block(&name) {
            eprintln!("Error executing block '{}': {}", name, e);
        } else {
            executed_blocks.insert(name);
        }
    }
    
    if !executed_blocks.is_empty() && CONFIG.lock().unwrap().verbose {
        println!("Auto-executed {} blocks", executed_blocks.len());
    }
    
    Ok(())
}

// Process question blocks without responses
fn answer_questions(executor: &mut MetaLanguageExecutor) -> Result<()> {
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
    
    // Process each question block
    for (name, _) in question_blocks {
        if let Err(e) = executor.execute_block(&name) {
            eprintln!("Error executing question block '{}': {}", name, e);
        } else {
            questions_answered += 1;
        }
    }
    
    if questions_answered > 0 && CONFIG.lock().unwrap().verbose {
        println!("Answered {} questions", questions_answered);
    }
    
    Ok(())
}

// Update the original file with execution results
fn update_file_with_results(file_path: &str, executor: &mut MetaLanguageExecutor) -> Result<()> {
    // Generate updated document content
    let updated_content = executor.update_document()
        .with_context(|| format!("Failed to update document content for {}", file_path))?;
    
    // Read the current file content
    let current_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file for update: {}", file_path))?;
    
    // Only write if content has changed
    if updated_content != current_content {
        if CONFIG.lock().unwrap().verbose {
            println!("Updating file with execution results: {}", file_path);
        }
        
        fs::write(file_path, updated_content)
            .with_context(|| format!("Failed to write updated content to {}", file_path))?;
    }
    
    Ok(())
}

// Handle a file system event from the watcher
fn handle_file_event(event: FileEvent) {
    let config = CONFIG.lock().unwrap();
    
    // Only process file modifications
    if event.event_type != FileEventType::Modified {
        return;
    }
    
    // Check if the file extension is one we care about
    let path = Path::new(&event.path);
    if let Some(ext) = path.extension() {
        let ext_str = format!(".{}", ext.to_string_lossy());
        if !config.watch_extensions.contains(&ext_str) {
            return;
        }
    } else {
        return; // No extension, not a file we want to process
    }
    
    // Process the modified file
    if config.verbose {
        println!("[{}] File changed: {}", Local::now().format("%H:%M:%S"), event.path);
    }
    
    if let Err(e) = process_file(&event.path) {
        eprintln!("Error processing file {}: {}", event.path, e);
    }
}

// Start the file watcher
fn start_file_watcher() -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = FileWatcher::new(tx);
    
    // Add watch paths
    let config = CONFIG.lock().unwrap();
    for path in &config.watch_paths {
        if let Err(e) = watcher.watch(path.clone()) {
            eprintln!("Error watching path {}: {}", path, e);
        } else if config.verbose {
            println!("Watching path: {}", path);
        }
    }
    
    // Handle events in a loop
    println!("Watching for file changes. Press Ctrl+C to exit.");
    drop(config); // Release the mutex lock
    
    loop {
        match rx.recv() {
            Ok(event) => handle_file_event(event),
            Err(e) => {
                eprintln!("File watcher error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

// Parse command-line arguments and configure the application
fn parse_args() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut config = CONFIG.lock().unwrap();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--watch" | "-w" => {
                config.watch_mode = true;
            },
            "--path" | "-p" => {
                if i + 1 < args.len() {
                    config.watch_paths = vec![args[i + 1].clone()];
                    i += 1;
                }
            },
            "--extensions" | "-e" => {
                if i + 1 < args.len() {
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
                    i += 1;
                }
            },
            "--no-auto-execute" => {
                config.auto_execute = false;
            },
            "--no-questions" => {
                config.answer_questions = false;
            },
            "--no-update" => {
                config.update_files = false;
            },
            "--verbose" | "-v" => {
                config.verbose = true;
            },
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            },
            // If not a known flag, assume it's a file to process
            _ if args[i].starts_with('-') => {
                return Err(anyhow!("Unknown option: {}", args[i]));
            },
            // Individual file to process
            _ => {
                // Store the file path for later processing
                return Ok(());
            }
        }
        i += 1;
    }
    
    Ok(())
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
    for file in files {
        if let Err(e) = process_file(file) {
            eprintln!("Error processing file {}: {}", file, e);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    // Parse command-line arguments
    if let Err(e) = parse_args() {
        eprintln!("Error parsing arguments: {}", e);
        print_usage();
        process::exit(1);
    }
    
    // Check for files to process
    let files: Vec<String> = env::args()
        .skip(1)
        .filter(|arg| !arg.starts_with('-') && arg != "--path" && arg != "--extensions")
        .collect();
    
    // Process specified files
    if !files.is_empty() {
        process_files(&files)?;
    }
    
    // Start file watcher if in watch mode
    let config = CONFIG.lock().unwrap();
    if config.watch_mode {
        drop(config); // Release the lock
        start_file_watcher()?;
    } else if files.is_empty() {
        // No files specified and not in watch mode, print usage
        print_usage();
    }
    
    Ok(())
}
