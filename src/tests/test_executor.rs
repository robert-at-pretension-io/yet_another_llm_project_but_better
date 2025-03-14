use std::fs;
use std::path::Path;

use crate::parser::parse_document;
use crate::executor::MetaLanguageExecutor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_references() {
        let test_file = "src/tests/test_variable_references.md";
        
        // Ensure the test file exists
        assert!(Path::new(test_file).exists(), "Test file not found");
        
        // Read the test file
        let content = fs::read_to_string(test_file).expect("Failed to read test file");
        
        // Create executor and process document
        let mut executor = MetaLanguageExecutor::new();
        executor.process_document(&content).expect("Failed to process document");
        
        // Verify blocks were executed
        assert!(executor.outputs.contains_key("generate_data"), "generate_data block not executed");
        assert!(executor.outputs.contains_key("process_data"), "process_data block not executed");
        assert!(executor.outputs.contains_key("final_output"), "final_output block not executed");
        
        // Verify dependencies were respected (execution order)
        let final_output = executor.outputs.get("final_output").expect("Missing final output");
        assert!(final_output.contains("Skills: 4"), "Expected 4 skills in final output");
        
        // Verify variable references were processed
        let process_data = executor.outputs.get("process_data").expect("Missing process_data output");
        assert!(process_data.contains("Meta-Language"), "Variable reference not processed correctly");
    }
}
