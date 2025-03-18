use crate::executor::error::ExecutorError;
use crate::executor::state::ExecutorState;
use crate::parser::Block;

/// Trait for block execution implementations
pub trait BlockRunner: Send + Sync {
    /// Check if this runner can execute the given block type
    fn can_execute(&self, block: &Block) -> bool;
    
    /// Execute the block and return its output
    fn execute(&self, block_name: &str, block: &Block, state: &mut ExecutorState) -> Result<String, ExecutorError>;
}

// We'll implement specific runners in separate modules
pub mod shell;
pub mod code;
pub mod conditional;
pub mod question;

/// Registry of block runners
pub struct RunnerRegistry {
    runners: Vec<Box<dyn BlockRunner>>,
}

impl RunnerRegistry {
    pub fn new() -> Self {
        let mut registry = Self { runners: Vec::new() };
        
        // Register all implemented runners
        registry.register(Box::new(shell::ShellRunner));
        registry.register(Box::new(code::PythonRunner));
        registry.register(Box::new(code::JavaScriptRunner));
        registry.register(Box::new(conditional::ConditionalRunner));
        registry.register(Box::new(question::QuestionRunner));
        
        registry
    }
    
    pub fn register(&mut self, runner: Box<dyn BlockRunner>) {
        self.runners.push(runner);
    }
    
    pub fn find_runner(&self, block: &Block) -> Option<&dyn BlockRunner> {
        self.runners.iter()
            .find(|runner| runner.can_execute(block))
            .map(|runner| runner.as_ref())
    }
}
