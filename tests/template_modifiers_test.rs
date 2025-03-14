use std::collections::HashMap;
use yet_another_llm_project_but_better::{parser, executor};
use yet_another_llm_project_but_better::parser::Block;

#[test]
fn test_template_with_modifiers() {
    println!("TEST: Creating template block directly");

    // Create the template block directly
    let mut template_block = Block::new("template", Some("test-template"), "This is a template with ${data-block}");
    template_block.add_modifier("requires", "data-block");
    template_block.add_modifier("cache", "true");
    
    // Check the block type
    println!("TEST: Template block_type: '{}'", template_block.block_type);
    assert_eq!(template_block.block_type, "template", "Incorrect block type");
    
    // Check the modifiers
    println!("TEST: Template has {} modifiers:", template_block.modifiers.len());
    for (k, v) in &template_block.modifiers {
        println!("TEST:   '{}' = '{}'", k, v);
    }
    
    // Verify specific modifiers
    assert!(template_block.has_modifier("requires"), "Missing 'requires' modifier");
    assert_eq!(template_block.get_modifier("requires"), Some(&"data-block".to_string()), "Incorrect 'requires' value");
    
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
