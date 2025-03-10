use regex::Regex;

// Process dependencies and requires modifiers
pub fn extract_dependencies(modifier_text: &str) -> Vec<(String, String)> {
    let mut modifiers = Vec::new();
    
    // Extract 'depends' modifier
    let depends_pattern = Regex::new(r"depends:([a-zA-Z0-9_-]+(?:,[a-zA-Z0-9_-]+)*)").unwrap();
    if let Some(cap) = depends_pattern.captures(modifier_text) {
        let deps_value = cap[1].to_string();
        // Handle multiple comma-separated dependencies
        if deps_value.contains(',') {
            // Keep the original string for reference
            modifiers.push(("depends".to_string(), deps_value.clone()));
            
            // Also create individual modifiers for each dependency
            for dep in deps_value.split(',') {
                modifiers.push((format!("depends:{}", dep.trim()), dep.trim().to_string()));
            }
        } else {
            modifiers.push(("depends".to_string(), deps_value));
        }
    }
    
    // Extract 'requires' modifier
    let requires_pattern = Regex::new(r"requires:([a-zA-Z0-9_-]+(?:,[a-zA-Z0-9_-]+)*)").unwrap();
    if let Some(cap) = requires_pattern.captures(modifier_text) {
        let reqs_value = cap[1].to_string();
        // Handle multiple comma-separated requirements
        if reqs_value.contains(',') {
            // Keep the original string for reference
            modifiers.push(("requires".to_string(), reqs_value.clone()));
            
            // Also create individual modifiers for each requirement
            for req in reqs_value.split(',') {
                modifiers.push((format!("requires:{}", req.trim()), req.trim().to_string()));
            }
        } else {
            modifiers.push(("requires".to_string(), reqs_value));
        }
    }
    
    modifiers
}
