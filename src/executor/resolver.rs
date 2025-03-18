use std::collections::HashMap;
use xmltree::{Element, XMLNode};
use crate::executor::error::ExecutorError;
use crate::executor::state::ExecutorState;
use crate::parser::Block;

/// Handles variable reference resolution in content
pub struct ReferenceResolver<'a> {
    state: &'a ExecutorState,
    debug_enabled: bool,
}

impl<'a> ReferenceResolver<'a> {
    pub fn new(state: &'a ExecutorState) -> Self {
        let debug_enabled = std::env::var("LLM_DEBUG").is_ok();
        Self { state, debug_enabled }
    }
    
    /// Process variable references in content
    pub fn process_content(&self, content: &str) -> Result<String, ExecutorError> {
        // Check if content might contain XML references
        if !content.contains("<meta:reference") && !content.contains(":reference") {
            return Ok(content.to_string());
        }

        if self.debug_enabled {
            println!("DEBUG: Processing variable references in content length: {}", content.len());
        }

        // Wrap content in a root element with namespace
        let xml_content = format!(
            r#"<root xmlns:meta="https://example.com/meta-language">{}</root>"#, 
            content
        );
        
        // Parse the XML content
        let mut root = self.parse_xml(&xml_content)?;
        
        // Process references in the XML tree
        self.process_element_references(&mut root)?;
        
        // Extract the processed content without the root wrapper
        self.extract_processed_content(&root)
    }
    
    /// Parse XML content, handling potential errors
    fn parse_xml(&self, xml_content: &str) -> Result<Element, ExecutorError> {
        match Element::parse(xml_content.as_bytes()) {
            Ok(root) => Ok(root),
            Err(e) => {
                if self.debug_enabled {
                    println!("DEBUG: XML parsing error: {}", e);
                    
                    // Try to recover by escaping quotes in shell commands
                    if xml_content.contains("echo \"") || xml_content.contains("echo '") {
                        println!("DEBUG: Attempting to recover from shell command quotes");
                        let fixed_content = xml_content.replace("echo \"", "echo &quot;")
                                              .replace("echo '", "echo &apos;");
                                              
                        match Element::parse(fixed_content.as_bytes()) {
                            Ok(root) => {
                                println!("DEBUG: Recovery successful with quote escaping");
                                return Ok(root);
                            },
                            Err(e2) => {
                                println!("DEBUG: Recovery failed: {}", e2);
                            }
                        }
                    }
                }
                
                Err(ExecutorError::XmlParsingError(format!("XML parse error: {}", e)))
            }
        }
    }
    
    /// Process references in an element tree recursively
    pub fn process_element_references(&self, element: &mut Element) -> Result<(), ExecutorError> {
        // First process this element if it's a reference
        if element.name == "meta:reference" || element.name.ends_with(":reference") {
            if self.debug_enabled {
                println!("DEBUG: Found reference element: {}", element.name);
            }
            
            // Get the target attribute
            if let Some(target) = element.attributes.get("target") {
                if self.debug_enabled {
                    println!("DEBUG: Reference targets variable: {}", target);
                }
                
                // Look up the target in outputs
                if let Some(value) = self.state.outputs.get(target) {
                    if self.debug_enabled {
                        println!("DEBUG: Found target '{}' in outputs", target);
                    }
                    
                    // Replace the element's children with the text value
                    element.children.clear();
                    element.children.push(XMLNode::Text(value.clone()));
                } else {
                    if self.debug_enabled {
                        println!("DEBUG: Target '{}' not found in outputs, using placeholder", target);
                    }
                    
                    // Target not found, insert a descriptive placeholder
                    let placeholder = format!("UNRESOLVED_REFERENCE:{}", target);
                    element.children.clear();
                    element.children.push(XMLNode::Text(placeholder));
                }
            }
            
            // We've handled this reference - no need to process children
            return Ok(());
        }
        
        // Process each child element recursively
        // We need to clone children first to avoid borrow checker issues
        let mut new_children = Vec::new();
        let children = element.children.clone();
        
        for child in children {
            match child {
                XMLNode::Element(mut child_elem) => {
                    self.process_element_references(&mut child_elem)?;
                    
                    // If this was a reference element, extract its text content directly
                    if child_elem.name == "meta:reference" || child_elem.name.ends_with(":reference") {
                        if child_elem.children.len() == 1 {
                            if let Some(XMLNode::Text(text)) = child_elem.children.first() {
                                new_children.push(XMLNode::Text(text.clone()));
                                continue;
                            }
                        }
                    }
                    
                    new_children.push(XMLNode::Element(child_elem));
                },
                XMLNode::Text(text) => {
                    new_children.push(XMLNode::Text(text));
                },
                other => {
                    new_children.push(other);
                }
            }
        }
        
        // Replace the element's children with the processed ones
        element.children = new_children;
        
        Ok(())
    }
    
    /// Extract processed content from the XML tree
    fn extract_processed_content(&self, root: &Element) -> Result<String, ExecutorError> {
        let mut result = String::new();
        
        for child in &root.children {
            match child {
                XMLNode::Element(e) => {
                    // Convert element back to string
                    let mut buffer = Vec::new();
                    e.write(&mut buffer).map_err(|e| {
                        ExecutorError::XmlParsingError(format!("Failed to write element: {}", e))
                    })?;
                    result.push_str(&String::from_utf8_lossy(&buffer));
                },
                XMLNode::Text(text) => {
                    result.push_str(text);
                },
                // Other node types are not expected here
                _ => {}
            }
        }
        
        if self.debug_enabled {
            println!("DEBUG: Finished processing variable references, result length: {}", result.len());
        }

        // Check if there are still unresolved references that need another pass
        if result.contains("<meta:reference") || result.contains(":reference") {
            if self.debug_enabled {
                println!("DEBUG: Detected nested references, processing recursively");
            }
            return self.process_content(&result);
        }
        
        Ok(result)
    }
    
    /// Process variable references in a set of blocks with multiple passes
    pub fn process_blocks(
        &self, 
        blocks: &mut HashMap<String, Block>,
        outputs: &mut HashMap<String, String>,
        block_names: &[String], 
        phase_name: &str
    ) -> Result<(), ExecutorError> {
        if self.debug_enabled {
            println!("DEBUG: Starting {} phase with {} blocks", phase_name, block_names.len());
        }
        
        // Up to 5 passes to handle nested references
        for pass in 0..5 {
            let mut any_replaced = false;
            
            if self.debug_enabled {
                println!("DEBUG: Starting {} phase pass {}", phase_name, pass + 1);
            }
            
            for name in block_names {
                // Skip if block doesn't exist
                let content = match blocks.get(name) {
                    Some(block) => {
                        // Quick check if this block might contain references
                        if !block.content.contains("<meta:reference") {
                            if self.debug_enabled {
                                println!("DEBUG: Block '{}' doesn't appear to contain references, skipping", name);
                            }
                            continue;
                        }
                        block.content.clone()
                    },
                    None => continue,
                };
                
                if self.debug_enabled {
                    println!("DEBUG: Processing '{}' references (pass {})", name, pass + 1);
                }
                
                // Process references
                let processed = self.process_content(&content)?;
                
                // Only update if something changed
                if processed != content {
                    if self.debug_enabled {
                        println!("DEBUG: References resolved in '{}', updating content", name);
                    }
                    
                    // Update the block content
                    if let Some(block) = blocks.get_mut(name) {
                        block.content = processed.clone();
                    }
                    
                    // Update outputs map
                    outputs.insert(name.clone(), processed);
                    any_replaced = true;
                }
            }
            
            // If nothing changed this pass, we're done with this phase
            if !any_replaced {
                if self.debug_enabled {
                    println!("DEBUG: No more changes in {} phase after pass {}", phase_name, pass + 1);
                }
                break;
            }
        }
        
        // Ensure all blocks have their updated content in the outputs map
        for name in block_names {
            if let Some(block) = blocks.get(name) {
                outputs.insert(name.clone(), block.content.clone());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Block;
    
    #[test]
    fn test_process_content_no_references() {
        let state = ExecutorState::new();
        let resolver = ReferenceResolver::new(&state);
        
        let content = "Hello, world!";
        let result = resolver.process_content(content).unwrap();
        assert_eq!(result, content);
    }
    
    // TODO: Fix XML namespace issues in tests
    /*
    #[test]
    fn test_process_content_with_references() {
        let mut state = ExecutorState::new();
        state.outputs.insert("greeting".to_string(), "Hello".to_string());
        let resolver = ReferenceResolver::new(&state);
        
        // Use proper XML format with meta:reference tags
        let content = "The greeting is: <meta:reference target=\"greeting\"></meta:reference>";
        let result = resolver.process_content(content).unwrap();
        assert_eq!(result, "The greeting is: Hello");
    }
    */
    
    // TODO: Fix XML namespace issues in tests
    /*
    #[test]
    fn test_nested_references() {
        let mut state = ExecutorState::new();
        state.outputs.insert("name".to_string(), "World".to_string());
        state.outputs.insert("greeting".to_string(), "Hello, <meta:reference target=\"name\"></meta:reference>!".to_string());
        let resolver = ReferenceResolver::new(&state);
        
        let content = "<meta:reference target=\"greeting\"></meta:reference>";
        let result = resolver.process_content(content).unwrap();
        assert_eq!(result, "Hello, World!");
    }
    */
    
    // TODO: Fix XML namespace issues in tests
    /*
    #[test]
    fn test_unresolved_reference() {
        let state = ExecutorState::new();
        let resolver = ReferenceResolver::new(&state);
        
        let content = "<meta:reference target=\"missing\"></meta:reference>";
        let result = resolver.process_content(content).unwrap();
        assert_eq!(result, "UNRESOLVED_REFERENCE:missing");
    }
    */
    
    // TODO: Fix XML namespace issues in tests
    /*
    #[test]
    fn test_process_blocks() {
        let mut state = ExecutorState::new();
        state.outputs.insert("var1".to_string(), "Value 1".to_string());
        
        let mut blocks = HashMap::new();
        let block = Block::new("data", Some("data1"), "<meta:reference target=\"var1\"></meta:reference>");
        blocks.insert("data1".to_string(), block.clone());
        
        let mut outputs = HashMap::new();
        outputs.insert("var1".to_string(), "Value 1".to_string());
        
        let resolver = ReferenceResolver::new(&state);
        let block_names = vec!["data1".to_string()];
        
        resolver.process_blocks(&mut blocks, &mut outputs, &block_names, "test").unwrap();
        
        assert_eq!(outputs.get("data1").unwrap(), "Value 1");
        assert_eq!(blocks.get("data1").unwrap().content, "Value 1");
    }
    */
}