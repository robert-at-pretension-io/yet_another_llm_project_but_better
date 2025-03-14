use std::collections::HashMap;
use meta_language::{parser, executor};
use meta_language::parser::blocks::Block;

#[test]
fn test_template_with_modifiers() {
    // Create a simple template with multiple modifiers
    let input = r#"[template name:test-template requires:data-block _type:custom cache:true]
This is a template with ${data-block}
[/template]"#;

    println!("TEST: Input template:\n{}", input);

    // Parse the template
    let result = parser::parse_document(input);
    assert!(result.is_ok(), "Failed to parse template: {:?}", result.err());

    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 1, "Expected 1 block, got {}", blocks.len());

    // Get the template block
    let template_block = &blocks[0];
    assert_eq!(template_block.block_type, "template:custom", "Expected template:custom block type");
    
    // Check the name
    assert_eq!(template_block.name, Some("test-template".to_string()), "Template name mismatch");
    
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
    let mut data_block = Block::new("data", Some("data-block"), "test data");
    executor.blocks.insert("data-block".to_string(), data_block);
    
    // Execute the template
    let result = executor.execute_block(&name);
    assert!(result.is_ok(), "Failed to execute template: {:?}", result.err());
    
    let output = result.unwrap();
    println!("TEST: Template execution result: '{}'", output);
    assert!(output.contains("test data"), "Template output doesn't contain data block content");
}
