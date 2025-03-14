use std::collections::HashMap;
use yet_another_llm_project_but_better::{parser, executor};
use yet_another_llm_project_but_better::parser::Block;

#[test]
fn test_template_with_modifiers() {
    println!("TEST: Parsing template block from text");

    // Create a template block using the parser
    let template_text = r#"
    [template:custom name:test-template requires:data-block cache:true]
    This is a template with ${data-block}
    [/template:custom]
    "#;
    
    // Parse the template
    let parsed_blocks = parser::parse_document(template_text).expect("Failed to parse template");
    
    // Find the template block
    let template_block = parsed_blocks.iter()
        .find(|b| b.name.as_ref().map_or(false, |n| n == "test-template"))
        .expect("Template block not found in parsed result")
        .clone();
    
    // Check the block type
    println!("TEST: Template block_type: '{}'", template_block.block_type);
    assert_eq!(template_block.block_type, "template:custom", "Incorrect block type");
    
    // Check the modifiers
    println!("TEST: Template has {} modifiers:", template_block.modifiers.len());
    for (k, v) in &template_block.modifiers {
        println!("TEST:   '{}' = '{}'", k, v);
    }
    
    // Verify specific modifiers
    assert!(template_block.has_modifier("requires"), "Missing 'requires' modifier");
    assert_eq!(template_block.get_modifier("requires"), Some(&"data-block".to_string()), "Incorrect 'requires' value");
    
    assert!(template_block.has_modifier("_type"), "Missing '_type' modifier");
    assert_eq!(template_block.get_modifier("_type"), Some(&"custom".to_string()), "Incorrect '_type' value");
    
    assert!(template_block.has_modifier("cache"), "Missing 'cache' modifier");
    assert_eq!(template_block.get_modifier("cache"), Some(&"true".to_string()), "Incorrect 'cache' value");
    
    // Test executor with the template
    let mut executor = executor::MetaLanguageExecutor::new();
    
    // Register the template block
    let name = template_block.name.as_ref().unwrap().clone();
    executor.blocks.insert(name.clone(), template_block.clone());
    
    // Add a data block for the dependency
    let data_block = Block::new("data", Some("data-block"), "test data");
    executor.blocks.insert("data-block".to_string(), data_block);
    
    // Execute the template
    let result = executor.execute_block(&name);
    assert!(result.is_ok(), "Failed to execute template: {:?}", result.err());
    
    let output = result.unwrap();
    println!("TEST: Template execution result: '{}'", output);
    assert!(output.contains("test data"), "Template output doesn't contain data block content");
}
