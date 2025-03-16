use std::collections::HashMap;
use serde_json;
use regex;

use crate::parser::Block;

/// Adds enhanced variable reference functionality to the MetaLanguageExecutor
impl crate::executor::MetaLanguageExecutor {
    /// Parse a variable reference with enhanced modifiers
    /// Format: variable-name:modifier1=value1,modifier2=value2
    pub fn parse_variable_reference(&self, var_ref: &str) -> (String, HashMap<String, String>) {
        let mut modifiers = HashMap::new();
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Parsing variable reference: '{}'", var_ref);
        }
        
        // Check if the variable reference contains modifiers
        if var_ref.contains(':') {
            let parts: Vec<&str> = var_ref.splitn(2, ':').collect();
            let var_name = parts[0].to_string();
            
            // Parse modifiers (format: key=value,key2=value2)
            if parts.len() > 1 {
                let modifiers_str = parts[1];
                
                if debug_enabled {
                    println!("DEBUG: Parsing modifiers string: '{}'", modifiers_str);
                }
                
                // Split by commas, but handle nested variable references
                let mut current_modifier = String::new();
                let mut in_nested_var = false;
                let mut brace_count = 0;
                
                for c in modifiers_str.chars() {
                    match c {
                        ',' if !in_nested_var && brace_count == 0 => {
                            // End of a modifier
                            self.parse_single_modifier(&current_modifier, &mut modifiers);
                            current_modifier.clear();
                        },
                        '$' => {
                            current_modifier.push(c);
                            if modifiers_str.chars().nth(current_modifier.len()) == Some('{') {
                                in_nested_var = true;
                            }
                        },
                        '{' if in_nested_var => {
                            current_modifier.push(c);
                            brace_count += 1;
                        },
                        '}' if in_nested_var => {
                            current_modifier.push(c);
                            brace_count -= 1;
                            if brace_count == 0 {
                                in_nested_var = false;
                            }
                        },
                        _ => current_modifier.push(c),
                    }
                }
                
                // Don't forget the last modifier
                if !current_modifier.is_empty() {
                    self.parse_single_modifier(&current_modifier, &mut modifiers);
                }
                
                return (var_name, modifiers);
            }
            
            return (var_name, modifiers);
        }
        
        // No modifiers, just return the variable name
        (var_ref.to_string(), modifiers)
    }
    
    /// Parse a single modifier in the format key=value
    fn parse_single_modifier(&self, modifier: &str, modifiers: &mut HashMap<String, String>) {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Parsing single modifier: '{}'", modifier);
        }
        
        if let Some(pos) = modifier.find('=') {
            let key = modifier[..pos].trim().to_string();
            let value = modifier[pos+1..].trim().to_string();
            
            // Handle nested variable references in modifier values
            let processed_value = if value.contains("${") && value.contains("}") {
                self.process_variable_references_internal(&value, &mut Vec::new())
            } else {
                value
            };
            
            modifiers.insert(key, processed_value);
            
            if debug_enabled {
                println!("DEBUG: Added modifier: '{}' = '{}'", key, processed_value);
            }
        }
    }
    
    /// Apply modifiers to a variable value
    pub fn apply_modifiers_to_variable(&self, var_name: &str, value: &str, modifiers: &HashMap<String, String>) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok() || 
                           std::env::var("LLM_DEBUG_VARS").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying modifiers to variable '{}' with {} modifiers", 
                     var_name, modifiers.len());
        }
        
        // Start with the original value
        let mut processed = value.to_string();
        
        // Check if we have a block with this name to get additional modifiers from
        if let Some(block) = self.blocks.get(var_name) {
            // Apply trim if specified in block
            processed = self.apply_trim(block, &processed);
            
            // Apply max_lines if specified in block
            processed = self.apply_max_lines(block, &processed);
        }
        
        // Apply format modifiers
        if let Some(format) = modifiers.get("format") {
            processed = self.apply_format(var_name, &processed, format);
        }
        
        // Apply limit modifier
        if let Some(limit_str) = modifiers.get("limit") {
            if let Ok(limit) = limit_str.parse::<usize>() {
                processed = self.apply_limit(&processed, limit);
            }
        }
        
        // Apply transformation modifiers
        if let Some(transform) = modifiers.get("transform") {
            processed = self.apply_transformation(&processed, transform);
        }
        
        // Apply highlighting modifier
        if let Some(highlight) = modifiers.get("highlight") {
            if highlight == "true" {
                processed = self.apply_highlighting(var_name, &processed);
            }
        }
        
        // Apply include modifiers
        processed = self.apply_include_modifiers(var_name, &processed, modifiers);
        
        // Apply conditional modifiers
        processed = self.apply_conditional_modifiers(var_name, &processed, modifiers);
        
        processed
    }
    
    /// Apply format modifier (markdown, json, code, plain)
    fn apply_format(&self, var_name: &str, content: &str, format_type: &str) -> String {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        
        if debug_enabled {
            println!("DEBUG: Applying format '{}' to variable '{}'", format_type, var_name);
        }
        
        match format_type {
            "markdown" => {
                // Convert to markdown format
                if let Some(block) = self.blocks.get(var_name) {
                    if block.block_type.contains("json") || content.trim().starts_with('{') {
                        // Convert JSON to markdown
                        format!("```json\n{}\n```\n\nFormat: markdown", content)
                    } else if block.block_type.contains("code") {
                        // Add code block formatting
                        let lang = block.block_type.split(':').nth(1).unwrap_or("text");
                        format!("```{}\n{}\n```", lang, content)
                    } else {
                        // Basic markdown formatting
                        format!("## {}\n\n{}", var_name, content)
                    }
                } else {
                    // Basic markdown formatting
                    format!("## {}\n\n{}", var_name, content)
                }
            },
            "json" => {
                // Ensure content is valid JSON or wrap it
                if content.trim().starts_with('{') || content.trim().starts_with('[') {
                    content.to_string()
                } else {
                    format!("{{ \"content\": \"{}\" }}", content.replace("\"", "\\\""))
                }
            },
            "code" => {
                // Format as code block
                if let Some(block) = self.blocks.get(var_name) {
                    let lang = if block.block_type.contains(':') {
                        block.block_type.split(':').nth(1).unwrap_or("text")
                    } else {
                        "text"
                    };
                    format!("```{}\n{}\n```", lang, content)
                } else {
                    format!("```\n{}\n```", content)
                }
            },
            "plain" => content.to_string(),
            _ => {
                // Unknown format, return as is
                if debug_enabled {
                    println!("DEBUG: Unknown format type: {}", format_type);
                }
                content.to_string()
            }
        }
    }
    
    /// Apply limit modifier to truncate content
    fn apply_limit(&self, content: &str, limit: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() <= limit {
            return content.to_string();
        }
        
        // Take only the first 'limit' lines
        let limited = lines.iter().take(limit).cloned().collect::<Vec<&str>>().join("\n");
        format!("{}\n...(truncated, showing {} of {} lines)", limited, limit, lines.len())
    }
    
    /// Apply transformation modifiers (uppercase, lowercase, substring)
    fn apply_transformation(&self, content: &str, transform: &str) -> String {
        match transform {
            "uppercase" => content.to_uppercase(),
            "lowercase" => content.to_lowercase(),
            transform if transform.starts_with("substring(") && transform.ends_with(")") => {
                // Parse substring parameters
                let params = transform.trim_start_matches("substring(").trim_end_matches(")");
                let parts: Vec<&str> = params.split(',').collect();
                
                if parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<usize>()) {
                        if start < content.len() {
                            let end = std::cmp::min(end, content.len());
                            return content.chars().skip(start).take(end - start).collect();
                        }
                    }
                }
                
                // If parsing failed or invalid range, return original
                content.to_string()
            },
            _ => content.to_string() // Unknown transformation
        }
    }
    
    /// Apply highlighting modifier for code blocks
    fn apply_highlighting(&self, var_name: &str, content: &str) -> String {
        if let Some(block) = self.blocks.get(var_name) {
            if block.block_type.starts_with("code:") {
                let language = block.block_type.split(':').nth(1).unwrap_or("text");
                return format!("```{}\n{}\n```", language, content);
            }
        }
        
        // If not a code block or language not specified, use plain code block
        format!("```\n{}\n```", content)
    }
    
    /// Apply include modifiers (include_code, include_results)
    fn apply_include_modifiers(&self, var_name: &str, content: &str, modifiers: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        
        // Handle include_code modifier
        if let Some(include_code) = modifiers.get("include_code") {
            if include_code == "true" {
                result = format!("Code:\n```\n{}\n```\n\n{}", content, result);
            }
        }
        
        // Handle include_results modifier
        if let Some(include_results) = modifiers.get("include_results") {
            if include_results == "true" {
                // Look for results block associated with this block
                for (result_name, block) in &self.blocks {
                    if block.block_type == "results" {
                        if let Some(for_block) = block.get_modifier("for") {
                            if for_block == var_name {
                                result = format!("{}\n\nResults:\n{}", result, block.content);
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Apply conditional modifiers (include_sensitive)
    fn apply_conditional_modifiers(&self, var_name: &str, content: &str, modifiers: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        
        // Handle include_sensitive modifier
        if let Some(condition_var) = modifiers.get("include_sensitive") {
            // Check if the condition variable is "true"
            if condition_var == "true" {
                // Include everything
                return result;
            } else if condition_var == "false" {
                // Filter out sensitive information
                if let Some(block) = self.blocks.get(var_name) {
                    if block.block_type.contains("json") {
                        // For JSON, we could implement a more sophisticated filter
                        // This is a simple implementation that just removes "sensitive_info" fields
                        if result.contains("\"sensitive_info\"") {
                            // Very simple approach - in a real implementation you'd want to parse the JSON properly
                            result = result.replace("\"sensitive_info\": {", "\"sensitive_info\": \"[REDACTED]\",");
                            // Find and remove the sensitive_info object content
                            let start_idx = result.find("\"sensitive_info\": {");
                            if let Some(start) = start_idx {
                                let mut brace_count = 0;
                                let mut end_idx = start;
                                
                                for (i, c) in result[start..].char_indices() {
                                    if c == '{' {
                                        brace_count += 1;
                                    } else if c == '}' {
                                        brace_count -= 1;
                                        if brace_count == 0 {
                                            end_idx = start + i + 1;
                                            break;
                                        }
                                    }
                                }
                                
                                if end_idx > start {
                                    result = format!(
                                        "{}\"sensitive_info\": \"[REDACTED]\"{}",
                                        &result[..start],
                                        &result[end_idx..]
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        
        result
    }
}
