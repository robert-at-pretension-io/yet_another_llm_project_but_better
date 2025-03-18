use crate::llm_client::LlmClient;
use crate::executor::error::ExecutorError;
use crate::executor::state::ExecutorState;
use crate::parser::Block;
use super::BlockRunner;

/// Question block runner for LLM API interactions
pub struct QuestionRunner;

impl BlockRunner for QuestionRunner {
    fn can_execute(&self, block: &Block) -> bool {
        block.block_type == "question"
    }
    
    fn execute(&self, _block_name: &str, block: &Block, state: &mut ExecutorState) 
        -> Result<String, ExecutorError> 
    {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        let question = &block.content;
        
        if debug_enabled {
            println!("DEBUG: Executing question block: {}", question);
        }
        
        // Check if we're in test mode
        let test_mode_env = std::env::var("LLM_TEST_MODE").unwrap_or_default();
        let is_test_mode = block.is_modifier_true("test_mode")
            || !test_mode_env.is_empty()
            || test_mode_env == "1"
            || test_mode_env.to_lowercase() == "true";
        
        if is_test_mode {
            if debug_enabled {
                println!("DEBUG: Test mode detected");
            }
            
            let test_response = if let Some(test_response) = block.get_modifier("test_response") {
                test_response.clone()
            } else {
                "This is a simulated response for testing purposes.".to_string()
            };
            
            return Ok(test_response);
        }
        
        // Create LLM client from block modifiers
        let llm_client = LlmClient::from_block_modifiers(&block.modifiers);
        
        if debug_enabled {
            println!("DEBUG: Created LLM client with provider: {:?}", llm_client.config.provider);
        }
        
        // Check if we have an API key
        if llm_client.config.api_key.is_empty() {
            return Err(ExecutorError::MissingApiKey(
                "No API key provided for LLM. Set via block modifier or environment variable."
                    .to_string(),
            ));
        }
        
        // Prepare the prompt
        let mut prompt = question.to_string();
        
        // Add system prompt if provided
        if let Some(system_prompt) = block.get_modifier("system_prompt") {
            prompt = format!("{}\n\n{}", system_prompt, prompt);
            
            if debug_enabled {
                println!("DEBUG: Added system prompt, new prompt length: {}", prompt.len());
            }
        }
        
        // Add context if provided
        if let Some(context_block) = block.get_modifier("context") {
            if let Some(context_content) = state.outputs.get(context_block) {
                if debug_enabled {
                    println!("DEBUG: Found context block '{}', length: {}", 
                         context_block, context_content.len());
                }
                
                prompt = format!("Context:\n{}\n\nQuestion:\n{}", context_content, prompt);
            }
        }
        
        if debug_enabled {
            println!("DEBUG: Final prompt length: {}", prompt.len());
            println!("DEBUG: Sending prompt to LLM API");
        }
        
        // Execute the LLM request
        match llm_client.send_prompt(&prompt) {
            Ok(response) => {
                if debug_enabled {
                    println!("DEBUG: Received successful response from LLM, length: {}", 
                            response.len());
                }
                
                // Store the response in appropriate blocks and outputs
                if let Some(name) = &block.name {
                    let response_block_name = format!("{}_response", name);
                    state.outputs.insert(response_block_name, response.clone());
                } else {
                    state.outputs.insert("question_response".to_string(), response.clone());
                }
                
                Ok(response)
            },
            Err(e) => {
                if debug_enabled {
                    println!("DEBUG: LLM API error: {}", e);
                }
                
                Err(ExecutorError::LlmApiError(e.to_string()))
            }
        }
    }
}