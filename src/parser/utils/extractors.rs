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
    println!("DEBUG: Extracting value from: '{}'", pair.as_str());
    
    // Try to get inner part
    if let Some(inner) = pair.clone().into_inner().next() {
        match inner.as_rule() {
            Rule::quoted_string => {
                // For a quoted string, extract content between quotes
                let s = inner.as_str();
                println!("DEBUG: Found quoted string: '{}'", s);
                if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                    s[1..s.len()-1].to_string()
                } else {
                    s.to_string()
                }
            },
            Rule::boolean | Rule::number => {
                // For booleans and numbers, just use the raw text
                let val = inner.as_str().to_string();
                println!("DEBUG: Found boolean/number: '{}'", val);
                val
            },
            Rule::block_reference => {
                // For block references, use the raw text
                let val = inner.as_str().to_string();
                println!("DEBUG: Found block reference: '{}'", val);
                val
            },
            _ => {
                // For any other rule type, extract as is
                let val = inner.as_str().to_string();
                println!("DEBUG: Found other type: '{:?}' with value '{}'", inner.as_rule(), val);
                val
            }
        }
    } else {
        // If no inner rules, the value might be after the colon as plain text
        // Try to extract directly from pair text
        let pair_str = pair.as_str().trim();
        
        // Debug what we're trying to process
        println!("DEBUG: No inner rule found, using raw text: '{}'", pair_str);
        
        // Return the raw value as a fallback
        pair_str.to_string()
    }
}

// Function to find and extract variable references from content
pub fn extract_variable_references(content: &str) -> Vec<String> {
    println!("extract_variable_references input: '{}'", content);
    let reference_regex = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
    
    let mut references = Vec::new();
    for cap in reference_regex.captures_iter(content) {
        let ref_name = cap[1].to_string();
        println!("  Found reference: '{}'", ref_name);
        references.push(ref_name);
    }
    
    println!("  Total references found: {}", references.len());
    references
}
