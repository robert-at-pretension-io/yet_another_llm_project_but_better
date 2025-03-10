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
    
    for modifier_pair in pair.into_inner() {
        if modifier_pair.as_rule() == Rule::modifier {
            let mut key = String::new();
            let mut value = String::new();
            
            for part in modifier_pair.into_inner() {
                match part.as_rule() {
                    Rule::modifier_key => {
                        key = part.as_str().to_string();
                    }
                    Rule::modifier_value => {
                        value = extract_modifier_value(part);
                    }
                    _ => {}
                }
            }
            
            if !key.is_empty() {
                modifiers.push((key, value));
            }
        }
    }
    
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
