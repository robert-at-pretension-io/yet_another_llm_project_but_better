use std::process::Command;
use crate::executor::error::ExecutorError;
use crate::executor::state::ExecutorState;
use crate::parser::Block;
use super::BlockRunner;

/// Runner for shell command blocks
pub struct ShellRunner;

impl BlockRunner for ShellRunner {
    fn can_execute(&self, block: &Block) -> bool {
        block.block_type == "shell"
    }
    
    fn execute(&self, _block_name: &str, block: &Block, _state: &mut ExecutorState) 
        -> Result<String, ExecutorError> 
    {
        // Get the content
        let content = &block.content;
        
        // Execute shell command
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", content]).output()?
        } else {
            Command::new("sh").args(&["-c", content]).output()?
        };

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(ExecutorError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}
