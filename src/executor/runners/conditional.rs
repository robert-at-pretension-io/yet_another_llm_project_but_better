use crate::executor::error::ExecutorError;
use crate::executor::state::ExecutorState;
use crate::executor::resolver::ReferenceResolver;
use crate::parser::Block;
use super::BlockRunner;

/// Conditional block execution runner
pub struct ConditionalRunner;

impl BlockRunner for ConditionalRunner {
    fn can_execute(&self, block: &Block) -> bool {
        block.block_type == "conditional"
    }
    
    fn execute(&self, block_name: &str, block: &Block, state: &mut ExecutorState) 
        -> Result<String, ExecutorError> 
    {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        // Check if we're in test mode
        let test_mode = block.modifiers.iter().any(|(k, v)| {
            k == "test_mode" && (v == "true" || v == "1" || v == "yes")
        });
        
        if test_mode {
            println!("DEBUG: Running conditional block in test mode");
            
            // Check if we have a test response
            if let Some(response) = block.get_modifier("test_response") {
                println!("DEBUG: Using test response: {}", response);
                return Ok(response.to_string());
            } else {
                // Default test response is the content
                println!("DEBUG: Returning content as test response");
                return Ok(block.content.clone());
            }
        }
        
        // Get the condition reference
        if let Some(condition_block) = block.get_modifier("if") {
            if debug_enabled {
                println!("DEBUG: Executing conditional block with condition: {}", condition_block);
            }
            
            // Check if the referenced block has results
            if let Some(condition_result) = state.outputs.get(condition_block) {
                let result = condition_result.trim().to_lowercase();
                
                // Print the output value to help debug
                if debug_enabled {
                    println!("DEBUG: Condition output for '{}' is: '{}'", condition_block, condition_result);
                    println!("DEBUG: Trimmed and lowercased: '{}'", result);
                }
                
                let condition_met = result == "true" || result == "1" || result == "yes";
                
                if debug_enabled {
                    println!("DEBUG: Condition '{}' evaluated to: {}", 
                             condition_block, condition_met);
                }
                
                if condition_met {
                    // Simply return the content without processing
                    // This avoids issues where the content might try to be executed as code
                    // by the processor
                    Ok(block.content.clone())
                } else {
                    // Skip processing
                    if debug_enabled {
                        println!("DEBUG: Condition not met, skipping block {}", block_name);
                    }
                    Ok(String::new())
                }
            } else {
                // Block hasn't been executed yet
                Err(ExecutorError::ReferenceResolutionFailed(
                    format!("Condition block '{}' has not been executed", condition_block)
                ))
            }
        } else {
            // No condition defined
            Ok(block.content.clone())
        }
    }
}