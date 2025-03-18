use std::io::Write;
use std::process::Command;
use tempfile;
use crate::executor::error::ExecutorError;
use crate::executor::state::ExecutorState;
use crate::parser::Block;
use super::BlockRunner;

/// Python code execution runner
pub struct PythonRunner;

impl BlockRunner for PythonRunner {
    fn can_execute(&self, block: &Block) -> bool {
        // Support both "code:python" and "code" with language="python"
        block.block_type == "code:python" || 
        (block.block_type == "code" && 
         block.get_modifier("language").map_or(false, |lang| lang == "python"))
    }
    
    fn execute(&self, block_name: &str, block: &Block, _state: &mut ExecutorState) 
        -> Result<String, ExecutorError> 
    {
        // Check if we're in test mode
        let test_mode = block.modifiers.iter().any(|(k, v)| {
            k == "test_mode" && (v == "true" || v == "1" || v == "yes")
        });
        
        if test_mode {
            println!("DEBUG: Running in test mode for block: {}", block_name);
            
            // Check if we have a test response
            if let Some(response) = block.get_modifier("test_response") {
                println!("DEBUG: Using test response: {}", response);
                return Ok(response.to_string());
            } else {
                println!("DEBUG: No test response provided, using empty string");
                return Ok("Test mode - no response".to_string());
            }
        }
        
        // If not in test mode, execute the code normally
        let code = &block.content;
        
        // Create a temporary Python file
        let mut tmpfile = tempfile::NamedTempFile::new()
            .map_err(|e| ExecutorError::IoError(e))?;
        let tmp_path = tmpfile.path().to_owned();

        // Write the code to the temporary file
        {
            let file = tmpfile.as_file_mut();
            file.write_all(code.as_bytes())
                .map_err(|e| ExecutorError::IoError(e))?;
            file.flush().map_err(|e| ExecutorError::IoError(e))?;
        }

        // Execute the Python file and capture its output
        println!(
            "DEBUG: Executing Python file using python3 at {:?}",
            tmp_path
        );
        
        let output = Command::new("python3")
            .arg(&tmp_path)
            .output()
            .map_err(|e| ExecutorError::IoError(e))?;
            
        println!(
            "DEBUG: Python execution completed with status: {:?}",
            output.status
        );

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            
            // For condition evaluation, clean up the output
            let cleaned = result.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
                
            Ok(cleaned)
        } else {
            Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}

/// JavaScript code execution runner
pub struct JavaScriptRunner;

impl BlockRunner for JavaScriptRunner {
    fn can_execute(&self, block: &Block) -> bool {
        // Support both "code:javascript" and "code" with language="javascript"
        block.block_type == "code:javascript" || 
        (block.block_type == "code" && 
         block.get_modifier("language").map_or(false, |lang| lang == "javascript"))
    }
    
    fn execute(&self, block_name: &str, block: &Block, _state: &mut ExecutorState) 
        -> Result<String, ExecutorError> 
    {
        // Check if we're in test mode
        let test_mode = block.modifiers.iter().any(|(k, v)| {
            k == "test_mode" && (v == "true" || v == "1" || v == "yes")
        });
        
        if test_mode {
            println!("DEBUG: Running in test mode for block: {}", block_name);
            
            // Check if we have a test response
            if let Some(response) = block.get_modifier("test_response") {
                println!("DEBUG: Using test response: {}", response);
                return Ok(response.to_string());
            } else {
                println!("DEBUG: No test response provided, using empty string");
                return Ok("Test mode - no response".to_string());
            }
        }
        
        // If not in test mode, execute the code normally
        let code = &block.content;
        
        // Create a temporary JS file
        let mut tmpfile = tempfile::NamedTempFile::new()
            .map_err(|e| ExecutorError::IoError(e))?;
        let tmp_path = tmpfile.path().to_owned();

        // Write the code to the temporary file
        {
            let file = tmpfile.as_file_mut();
            file.write_all(code.as_bytes())
                .map_err(|e| ExecutorError::IoError(e))?;
            file.flush().map_err(|e| ExecutorError::IoError(e))?;
        }

        // Execute the JavaScript file with Node.js
        let output = Command::new("node")
            .arg(&tmp_path)
            .output()
            .map_err(|e| ExecutorError::IoError(e))?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(result.trim().to_string())
        } else {
            Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}