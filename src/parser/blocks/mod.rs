use crate::parser::Rule;

// Basic block representation
#[derive(Debug, Clone)]
pub struct Block {
    pub block_type: String,
    pub name: Option<String>,
    pub modifiers: Vec<(String, String)>,
    pub content: String,
    pub children: Vec<Block>,
}

impl Block {
    // Create a new block
    pub fn new(block_type: &str, name: Option<&str>, content: &str) -> Self {
        Self {
            block_type: block_type.to_string(),
            name: name.map(|s| s.to_string()),
            modifiers: Vec::new(),
            content: content.to_string(),
            children: Vec::new(),
        }
    }
    
    // Add a modifier to the block
    pub fn add_modifier(&mut self, key: &str, value: &str) {
        self.modifiers.push((key.to_string(), value.to_string()));
    }
    
    // Check if the block has a specific modifier
    pub fn has_modifier(&self, key: &str) -> bool {
        self.modifiers.iter().any(|(k, _)| k == key)
    }
    
    // Get the value of a modifier
    pub fn get_modifier(&self, key: &str) -> Option<&String> {
        self.modifiers.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }
    
    // Check if a modifier has a truthy value (true, yes, 1)
    pub fn is_modifier_true(&self, key: &str) -> bool {
        if let Some(value) = self.get_modifier(key) {
            let value = value.to_lowercase();
            value == "true" || value == "yes" || value == "1"
        } else {
            false
        }
    }
    
    // Get a modifier value as f64
    pub fn get_modifier_as_f64(&self, key: &str) -> Option<f64> {
        self.get_modifier(key)
            .and_then(|v| v.parse::<f64>().ok())
    }
    
    // Add a child block
    pub fn add_child(&mut self, child: Block) {
        self.children.push(child);
    }
}

// Process a single block
pub fn process_block(pair: pest::iterators::Pair<Rule>) -> Option<Block> {
    // Get the first inner pair which should be the specific block type
    let inner_pairs = pair.into_inner();
    
    for block_pair in inner_pairs {
        match block_pair.as_rule() {
            Rule::question_block => return Some(process_question_block(block_pair)),
            Rule::code_block => return Some(process_code_block(block_pair)),
            Rule::data_block => return Some(process_data_block(block_pair)),
            Rule::shell_block => return Some(process_shell_block(block_pair)),
            Rule::api_block => return Some(process_api_block(block_pair)),
            Rule::variable_block => return Some(process_variable_block(block_pair)),
            Rule::secret_block => return Some(process_secret_block(block_pair)),
            Rule::template_block => return Some(process_template_block(block_pair)),
            Rule::template_invocation_block => return Some(process_template_invocation(block_pair)),
            Rule::error_block => return Some(process_error_block(block_pair)),
            Rule::visualization_block => return Some(process_visualization_block(block_pair)),
            Rule::preview_block => return Some(process_preview_block(block_pair)),
            Rule::response_block => return Some(process_response_block(block_pair)),
            Rule::filename_block => return Some(process_filename_block(block_pair)),
            Rule::memory_block => return Some(process_memory_block(block_pair)),
            Rule::section_block => return Some(process_section_block(block_pair)),
            Rule::conditional_block => return Some(process_conditional_block(block_pair)),
            Rule::results_block => return Some(process_results_block(block_pair)),
            Rule::error_results_block => return Some(process_error_results_block(block_pair)),
            _ => {
                eprintln!("Unhandled block type: {:?}", block_pair.as_rule());
                return None;
            }
        }
    }
    
    None
}

// Let's group the individual block processing functions into related files
mod question_response;
mod code_exec;
mod data_management;
mod templates;
mod utility;
mod results;

// Re-export the processing functions
pub use question_response::{process_question_block, process_response_block};
pub use code_exec::{process_code_block, process_shell_block, process_api_block};
pub use data_management::{process_data_block, process_variable_block, process_secret_block, process_filename_block, process_memory_block};
pub use templates::{process_template_block, process_template_invocation};
pub use utility::{process_error_block, process_visualization_block, process_preview_block, process_section_block, process_conditional_block};
pub use results::{process_results_block, process_error_results_block};
