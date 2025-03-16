use crate::parser::Rule;
use crate::parser::modifiers::extract_modifiers_from_text;

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
    
    // First try to extract using the structured approach
    for modifier_pair in pair.clone().into_inner() {
        if modifier_pair.as_rule() == Rule::modifier {
            let mut key = String::new();
            let mut value = String::new();
            
            // Debug: Print the raw modifier pair
            println!("DEBUG: Modifier pair: '{}'", modifier_pair.as_str());
            
            for part in modifier_pair.clone().into_inner() {
                match part.as_rule() {
                    Rule::modifier_key => {
                        key = part.as_str().to_string();
                        println!("DEBUG: Found modifier key: '{}'", key);
                    }
                    Rule::modifier_value => {
                        value = extract_modifier_value(part.clone());
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
    
    // If we didn't extract any modifiers using the structured approach,
    // or if we're dealing with a complex case, also try the regex approach
    if modifiers.is_empty() || pair.as_str().contains("\"") {
        let text_modifiers = extract_modifiers_from_text(pair.as_str());
        for (key, value) in text_modifiers {
            // Only add if we don't already have this key
            if !modifiers.iter().any(|(k, _)| k == &key) {
                println!("DEBUG: Adding text-extracted modifier: '{}' = '{}'", key, value);
                modifiers.push((key, value));
            }
        }
    }
    
    println!("DEBUG: Extracted {} modifiers", modifiers.len());
    modifiers
}

// Extract the value from a modifier_value pair
fn extract_modifier_value(pair: pest::iterators::Pair<Rule>) -> String {
    println!("DEBUG: Extracting value from: '{}'", pair.as_str());
    
    // Check if the raw text contains quotes
    let raw_text = pair.as_str();
    if raw_text.contains('"') {
        // Try to extract quoted content directly from the raw text
        if let Some(start) = raw_text.find('"') {
            if let Some(end) = raw_text[start+1..].find('"') {
                let quoted_value = &raw_text[start+1..start+1+end];
                println!("DEBUG: Extracted quoted value directly: '{}'", quoted_value);
                return quoted_value.to_string();
            }
        }
    }
    
    // Try to get inner part
    if let Some(inner) = pair.clone().into_inner().next() {
        match inner.as_rule() {
            Rule::quoted_string => {
                // For a quoted string, extract content between quotes
                let s = inner.as_str();
                println!("DEBUG: Found quoted string: '{}'", s);
                if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                    return s[1..s.len()-1].to_string();
                } else {
                    return s.to_string();
                }
            },
            Rule::boolean | Rule::number => {
                // For booleans and numbers, just use the raw text
                let val = inner.as_str().to_string();
                println!("DEBUG: Found boolean/number: '{}'", val);
                return val;
            },
            Rule::block_reference => {
                // For block references, use the raw text
                let val = inner.as_str().to_string();
                println!("DEBUG: Found block reference: '{}'", val);
                return val;
            },
            _ => {
                // For any other rule type, extract as is
                let val = inner.as_str().to_string();
                println!("DEBUG: Found other type: '{:?}' with value '{}'", inner.as_rule(), val);
                return val;
            }
        }
    } else {
        // If no inner rules, the value might be after the colon as plain text
        // Try to extract directly from pair text
        let pair_str = pair.as_str().trim();
        
        // Debug what we're trying to process
        println!("DEBUG: No inner rule found, using raw text: '{}'", pair_str);
        
        // Check if it's a quoted string in the raw text
        if pair_str.starts_with('"') && pair_str.ends_with('"') && pair_str.len() >= 2 {
            let unquoted = &pair_str[1..pair_str.len()-1];
            println!("DEBUG: Unquoted value: '{}'", unquoted);
            return unquoted.to_string();
        }
        
        // Return the raw value as a fallback
        return pair_str.to_string();
    }
}
    
// Extract variable references from content (variables in the format <meta:reference target="name" /> or __META_REFERENCE__name)
pub fn extract_variable_references(content: &str) -> Vec<String> {
    // Check if debugging is enabled
    let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
    
    let mut references = Vec::new();
    
    if debug_enabled {
        println!("DEBUG: Extracting variable references from content:");
        println!("DEBUG: ===== CONTENT START =====");
        println!("{}", if content.len() > 500 { 
            format!("{}... (truncated, total length: {})", &content[..500], content.len()) 
        } else { 
            content 
        });
        println!("DEBUG: ===== CONTENT END =====");
    }
    
    // Look for both XML references and our special markers
    if !content.contains("<meta:reference") && !content.contains("__META_REFERENCE__") {
        if debug_enabled {
            println!("DEBUG: No references found in content, returning empty list");
        }
        return references;
    }
    
    // First check for our special markers from the XML parser
    let re_marker = regex::Regex::new(r"__META_REFERENCE__([a-zA-Z0-9_-]+)").unwrap();
    for cap in re_marker.captures_iter(content) {
        let var_name = &cap[1];
        let start_pos = cap.get(0).unwrap().start();
        let end_pos = cap.get(0).unwrap().end();
        
        references.push(var_name.to_string());
        
        if debug_enabled {
            println!("DEBUG: Found marker reference at positions {}-{} for variable: {}", 
                     start_pos, end_pos, var_name);
        }
    }
    
    // Also handle any XML references that might still be present
    let mut start_pos = 0;
    while let Some(pos) = content[start_pos..].find("<meta:reference") {
        // Adjust position to be relative to the original string
        let tag_start = start_pos + pos;
        
        // Find the end of this tag
        if let Some(end_pos) = content[tag_start..].find("/>") {
            let tag_end = tag_start + end_pos + 2; // +2 for the "/>" itself
            let tag = &content[tag_start..tag_end];
            
            if debug_enabled {
                println!("DEBUG: Found XML reference at positions {}-{}: {}", 
                         tag_start, tag_end, tag);
            }
            
            // Extract the target attribute
            if let Some(target_start) = tag.find("target=\"") {
                let value_start = target_start + 8; // 8 is the length of 'target="'
                if let Some(value_end) = tag[value_start..].find('"') {
                    let target = &tag[value_start..value_start + value_end];
                    references.push(target.to_string());
                    
                    if debug_enabled {
                        println!("DEBUG: Extracted target: {} from XML reference", target);
                    }
                } else if debug_enabled {
                    println!("DEBUG: Failed to find closing quote for target attribute");
                }
            } else if debug_enabled {
                println!("DEBUG: No target attribute found in XML reference: {}", tag);
            }
            
            // Move start position for the next iteration
            start_pos = tag_end;
        } else {
            // No end tag found, break the loop
            if debug_enabled {
                println!("DEBUG: No closing tag found for XML reference starting at position {}", tag_start);
            }
            break;
        }
    }
    
    if debug_enabled {
        println!("DEBUG: Found a total of {} references: {:?}", references.len(), references);
    }
    
    references
}
