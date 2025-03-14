use std::fs;
use std::path::Path;
use std::env;

use yet_another_llm_project_but_better::parser::parse_document;
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;

/// Test harness for testing meta-language documents
#[derive(Default)]
pub struct DocumentTestHarness {
    executor: MetaLanguageExecutor,
}

impl DocumentTestHarness {
    pub fn new() -> Self {
        Self {
            executor: MetaLanguageExecutor::new(),
        }
    }
    
    /// Process a document string, returning success/failure and prints outputs
    pub fn test_document(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("======= Testing Document =======");
        
        // Parse the document
        println!("Parsing document...");
        let blocks = parse_document(content)?;
        println!("Found {} blocks", blocks.len());
        
        // Process with executor
        println!("Executing blocks...");
        self.executor.process_document(content)?;
        
        // Print block outputs
        println!("\n=== Block Outputs ===");
        for (name, output) in &self.executor.outputs {
            println!("--- {} ---", name);
            if output.lines().count() > 10 {
                // Print truncated output for large results
                for line in output.lines().take(10) {
                    println!("{}", line);
                }
                println!("... (truncated, total lines: {})", output.lines().count());
            } else {
                println!("{}", output);
            }
            println!();
        }
        
        // Generate updated document
        let _updated = self.executor.update_document()?;
        
        println!("=== Test Completed Successfully ===");
        Ok(())
    }
    
    /// Test a document file
    pub fn test_document_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(path).exists() {
            return Err(format!("File not found: {}", path).into());
        }
        
        println!("Testing document file: {}", path);
        let content = fs::read_to_string(path)?;
        self.test_document(&content)
    }
    
    /// Run document tests from a directory
    pub fn test_documents_in_directory(&mut self, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(dir).exists() || !Path::new(dir).is_dir() {
            return Err(format!("Directory not found: {}", dir).into());
        }
        
        println!("Testing documents in directory: {}", dir);
        let entries = fs::read_dir(dir)?;
        
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let path_str = path.to_string_lossy().to_string();
                println!("\n===== Testing {} =====", path_str);
                
                match self.test_document_file(&path_str) {
                    Ok(_) => {
                        println!("✅ PASS: {}", path_str);
                        success_count += 1;
                    },
                    Err(e) => {
                        println!("❌ FAIL: {} - Error: {}", path_str, e);
                        failure_count += 1;
                    }
                }
            }
        }
        
        println!("\n===== Test Results =====");
        println!("Passed: {}", success_count);
        println!("Failed: {}", failure_count);
        println!("Total: {}", success_count + failure_count);
        
        if failure_count > 0 {
            Err(format!("{} tests failed", failure_count).into())
        } else {
            Ok(())
        }
    }
}

// Make the DocumentTestHarness accessible to other modules
pub use DocumentTestHarness;

// Command line test runner
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <file_or_directory_path>", args[0]);
        return Ok(());
    }
    
    let path = &args[1];
    let mut harness = DocumentTestHarness::new();
    
    if Path::new(path).is_dir() {
        harness.test_documents_in_directory(path)?;
    } else {
        harness.test_document_file(path)?;
    }
    
    Ok(())
}
