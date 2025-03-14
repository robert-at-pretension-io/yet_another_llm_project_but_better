use std::path::Path;
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the tests directory if it doesn't exist
    let test_dir = "src/tests";
    if !Path::new(test_dir).exists() {
        std::fs::create_dir_all(test_dir)?;
    }
    
    // Create a test harness
    let mut harness = yet_another_llm_project_but_better::bin::test_harness::DocumentTestHarness::new();
    
    // Run tests in the tests directory
    println!("Running tests from directory: {}", test_dir);
    harness.test_documents_in_directory(test_dir)?;
    
    println!("All tests passed!");
    Ok(())
}
