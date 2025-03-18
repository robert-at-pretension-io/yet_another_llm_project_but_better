use std::time::Duration;
use crate::parser::Block;

/// Cache management functionality for the executor
pub struct CacheManager;

impl CacheManager {
    /// Check if a block's result should be cached
    pub fn is_cacheable(block: &Block) -> bool {
        // First check if caching is globally disabled via environment variable
        if let Ok(cache_disabled) = std::env::var("LLM_NO_CACHE") {
            if cache_disabled == "1" || cache_disabled.to_lowercase() == "true" {
                return false;
            }
        }
        
        // Check if there's an explicit never-cache modifier
        if block.modifiers.iter().any(|(key, value)| {
            key == "never-cache" && (value == "true" || value == "yes" || value == "1" || value == "on")
        }) {
            return false;
        }

        // For Python code blocks, we require explicit opt-in for caching
        if block.block_type == "code:python" {
            return block.modifiers.iter().any(|(key, value)| {
                key == "cache_result"
                    && (value == "true" || value == "yes" || value == "1" || value == "on")
            });
        }
        
        // By default, we want to cache results for other code types, shell, and API blocks
        let cacheable_types = ["code", "shell", "api", "code:javascript", "code:rust"];
        if cacheable_types.iter().any(|&t| block.block_type == t || block.block_type.starts_with(t)) {
            // Only avoid caching if there's an explicit "cache_result=false"
            return !block.modifiers.iter().any(|(key, value)| {
                key == "cache_result"
                    && (value == "false" || value == "no" || value == "0" || value == "off")
            });
        }

        // For other blocks, explicit opt-in is required
        block.modifiers.iter().any(|(key, value)| {
            key == "cache_result"
                && (value == "true" || value == "yes" || value == "1" || value == "on")
        })
    }

    /// Get timeout duration for a block
    pub fn get_timeout(block: &Block) -> Duration {
        // First check block modifiers
        for (key, value) in &block.modifiers {
            if key == "timeout" {
                if let Ok(seconds) = value.parse::<u64>() {
                    return Duration::from_secs(seconds);
                }
            }
        }

        // Then check environment variable
        if let Ok(timeout_str) = std::env::var("LLM_TIMEOUT") {
            if let Ok(seconds) = timeout_str.parse::<u64>() {
                return Duration::from_secs(seconds);
            }
        }

        // Default timeout (10 minutes)
        Duration::from_secs(600)
    }
}
