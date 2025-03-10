mod dependency;

pub use dependency::extract_dependencies;

// Extract modifiers from block definitions
pub fn extract_modifiers_from_text(text: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    
    // Handle name modifier separately as it's common pattern
    if let Some(name_start) = text.find("name:") {
        let name_start = name_start + 5; // skip "name:"
        let name_end = text[name_start..].find(' ')
            .map(|pos| name_start + pos)
            .unwrap_or(text.len());
        
        let name = text[name_start..name_end].trim();
        if !name.is_empty() {
            result.push(("name".to_string(), name.to_string()));
        }
    }
    
    // Extract depends and requires modifiers
    let deps = dependency::extract_dependencies(text);
    result.extend(deps);
    
    // Extract format modifier (common in data blocks)
    if let Some(format_start) = text.find("format:") {
        let format_start = format_start + 7; // skip "format:"
        let format_end = text[format_start..].find(' ')
            .map(|pos| format_start + pos)
            .unwrap_or(text.len());
        
        let format = text[format_start..format_end].trim();
        if !format.is_empty() {
            result.push(("format".to_string(), format.to_string()));
        }
    }
    
    // Parse other key:value modifiers
    for part in text.split_whitespace() {
        if part.contains(':') && !part.starts_with("name:") && 
           !part.starts_with("depends:") && !part.starts_with("requires:") && 
           !part.starts_with("format:") {
            
            let key_value: Vec<&str> = part.split(':').collect();
            if key_value.len() >= 2 {
                let key = key_value[0].trim();
                let value = key_value[1].trim().trim_matches('"');
                result.push((key.to_string(), value.to_string()));
            }
        }
    }
    
    result
}

// Extract modifier value from a string
pub fn extract_modifier_value(text: &str) -> String {
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        text[1..text.len()-1].to_string()
    } else {
        text.to_string()
    }
}
