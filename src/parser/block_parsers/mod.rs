// Module containing specialized parsers for different block types
mod communication_blocks;
mod executable_blocks;
mod data_blocks;
mod control_blocks;
mod template_blocks;
mod results_blocks;

// Re-export block parsers
pub use communication_blocks::*;
pub use executable_blocks::*;
pub use data_blocks::*;
pub use control_blocks::*;
pub use template_blocks::*;
pub use results_blocks::*;
