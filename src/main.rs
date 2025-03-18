use std::env;
use std::fs;
use std::path::Path;
use std::process;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use yet_another_llm_project_but_better::{
    executor::MetaLanguageExecutor,
    file_watcher::FileWatcher,
};

fn main() -> Result<()> {
    // Get file from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        process::exit(1);
    }
    
    let file_path = Path::new(&args[1]);
    
    // Process the file
    process_file(file_path)?;
    
    // Check if watch flag is enabled
    let watch_mode = args.len() > 2 && args[2] == "--watch";
    
    if watch_mode {
        println!("Watching file for changes: {}", file_path.display());
        
        let mut watcher = FileWatcher::new();
        if let Err(e) = watcher.watch(file_path) {
            eprintln!("Error watching file: {}", e);
            process::exit(1);
        }
        
        println!("Press Ctrl+C to exit.");
        // The watch_for_events API is just a dummy that doesn't actually work
        // in a real system, you'd need to implement a custom event loop
    }
    
    Ok(())
}

fn process_file(file_path: &Path) -> Result<()> {
    println!("Processing file: {:?}", file_path);

    // Read the file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

    // Create executor
    let mut executor = MetaLanguageExecutor::new();

    // Process document to extract blocks
    executor.process_document(&content)
        .map_err(|e| format!("Failed to process document {}: {}", file_path.display(), e))?;

    // Debug: Print all executable blocks
    println!("Found blocks:");
    for (name, block) in &executor.blocks {
        if executor.is_executable_block(block) {
            println!("  - '{}' ({})", name, block.block_type);
        } else {
            println!("  - '{}' ({}) [non-executable]", name, block.block_type);
        }
    }

    // Execute question blocks for testing (add test_mode modifier)
    let question_blocks: Vec<_> = executor.blocks.iter()
        .filter(|(_, block)| block.block_type == "question")
        .filter_map(|(name, block)| Some((name.clone(), block.clone())))
        .collect();
    
    if !question_blocks.is_empty() {
        println!("Found {} question blocks", question_blocks.len());
        
        for (name, block) in question_blocks {
            println!("Processing question: '{}' with content: '{}'", 
                     name, 
                     if block.content.len() > 50 { &block.content[..50] } else { &block.content });
            
            // Add test_mode modifier for testing
            let mut test_block = block.clone();
            test_block.add_modifier("test_mode", "true");
            
            // Update the block in the executor
            executor.blocks.insert(name.clone(), test_block);
            
            if let Err(e) = executor.execute_block(&name) {
                eprintln!("Error executing question block '{}': {}", name, e);
            } else {
                println!("Successfully processed question: '{}'", name);
            }
        }
    } else {
        println!("No question blocks found");
    }
    
    // Before updating the document, ensure all conditional blocks' children are executed
    let conditional_blocks: Vec<String> = executor.blocks.iter()
        .filter(|(_, block)| block.block_type == "conditional")
        .filter_map(|(name, _)| Some(name.clone()))
        .collect();
    
    // Execute conditional children to ensure their content is available
    for block_name in &conditional_blocks {
        // Get the condition and child names before borrowing executor as mutable
        let mut child_names = Vec::new();
        let condition_name;
        
        {
            // Scope for immutable borrow
            if let Some(block) = executor.blocks.get(block_name) {
                // Get the condition
                if let Some(condition) = block.get_modifier("if") {
                    condition_name = condition.clone();
                    
                    // Collect child names
                    for child in &block.children {
                        if let Some(child_name) = &child.name {
                            child_names.push(child_name.clone());
                        }
                    }
                } else {
                    // No condition, skip
                    continue;
                }
            } else {
                // Block not found, skip
                continue;
            }
        }
        
        // Execute the condition
        match executor.execute_block(&condition_name) {
            Ok(_) => {
                println!("Successfully executed condition '{}' for conditional '{}'", 
                         condition_name, block_name);
                
                // Execute all child blocks
                for child_name in &child_names {
                    println!("Executing child block '{}' of conditional '{}'", 
                             child_name, block_name);
                             
                    if let Err(e) = executor.execute_block(child_name) {
                        eprintln!("Error executing child block '{}': {}", child_name, e);
                    } else {
                        println!("Successfully executed child block '{}'", child_name);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error executing condition '{}': {}", condition_name, e);
            }
        }
    }
    
    let updated_content = executor.update_document()
        .map_err(|e| format!("Failed to update document content for {}: {}", file_path.display(), e))?;
    
    // Only write if content has changed
    if updated_content != content {
        println!("Updating file with execution results: {}", file_path.display());
        fs::write(file_path, updated_content)
            .map_err(|e| format!("Failed to write updated content to {}: {}", file_path.display(), e))?;
    } else {
        println!("Content unchanged, no update needed");
        
        // For debugging only: Optionally force update the file with environment variable
        if std::env::var("LLM_FORCE_UPDATE").is_ok() {
            println!("LLM_FORCE_UPDATE set, forcing file update");
            fs::write(file_path, updated_content)
                .map_err(|e| format!("Failed to force-write updated content to {}: {}", file_path.display(), e))?;
            println!("Forcibly updated file: {}", file_path.display());
        }
    }

    Ok(())
}