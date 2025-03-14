use crate::parser::Rule;

// Helper function to extract the name from a name_attr pair
pub fn extract_name(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::block_name {
            return Some(inner_pair.as_str().to_string());
        }
    }
    None
}

// Extract modifiers from a modifiers pair
pub fn extract_modifiers(pair: pest::iterators::Pair<Rule>) -> Vec<(String, String)> {
    let mut modifiers = Vec::new();
    
    // Debug: Print the raw modifier text
    println!("DEBUG: Raw modifiers text: '{}'", pair.as_str());
    
    for modifier_pair in pair.into_inner() {
        if modifier_pair.as_rule() == Rule::modifier {
            let mut key = String::new();
            let mut value = String::new();
            
            // Debug: Print the raw modifier pair
            println!("DEBUG: Modifier pair: '{}'", modifier_pair.as_str());
            
            for part in modifier_pair.into_inner() {
                match part.as_rule() {
                    Rule::modifier_key => {
                        key = part.as_str().to_string();
                        println!("DEBUG: Found modifier key: '{}'", key);
                    }
                    Rule::modifier_value => {
                        value = extract_modifier_value(part);
                        println!("DEBUG: Found modifier value: '{}'", value);
                    }
                    _ => {
                        println!("DEBUG: Unknown modifier part: '{}'", part.as_str());
                    }
                }
            }
            
            if !key.is_empty() {
                println!("DEBUG: Adding modifier: '{}' = '{}'", key, value);
                modifiers.push((key, value));
            }
        } else {
            println!("DEBUG: Non-modifier rule found: '{:?}' with text '{}'", 
                     modifier_pair.as_rule(), modifier_pair.as_str());
        }
    }
    
    println!("DEBUG: Extracted {} modifiers", modifiers.len());
    modifiers
}

// Extract the value from a modifier_value pair
fn extract_modifier_value(pair: pest::iterators::Pair<Rule>) -> String {
    if let Some(inner) = pair.into_inner().next() {
        match inner.as_rule() {
            Rule::quoted_string => {
                let s = inner.as_str();
                // Remove the quotes
                if s.len() >= 2 {
                    s[1..s.len()-1].to_string()
                } else {
                    s.to_string()
                }
            }
            _ => inner.as_str().to_string(),
        }
    } else {
        "".to_string() // Return empty string if no inner value
    }
}

// Function to find and extract variable references from content
pub fn extract_variable_references(content: &str) -> Vec<String> {
    let mut references = Vec::new();
    let re = regex::Regex::new(r"\$\{([a-zA-Z0-9_-]+)\}").unwrap();
    
    for cap in re.captures_iter(content) {
        references.push(cap[1].to_string());
    }
    
    references
}
