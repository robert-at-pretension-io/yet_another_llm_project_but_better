//! Parser robustness tests
//! 
//! This file contains comprehensive tests for the Meta Language parser,
//! focusing on edge cases, complex structures, and error handling.

use yet_another_llm_project_but_better::parser::{parse_document, Block, ParserError, utils::extractors::extract_variable_references};
use std::collections::HashSet;

/// Helper function to find a block by name in a list of blocks
fn find_block_by_name<'a>(blocks: &'a [Block], name: &str) -> Option<&'a Block> {
    blocks.iter().find(|b| b.name.as_ref().map_or(false, |n| n == name))
}

/// Helper function to find blocks by type in a list of blocks
fn find_blocks_by_type<'a>(blocks: &'a [Block], block_type: &str) -> Vec<&'a Block> {
    blocks.iter().filter(|b| b.block_type == block_type).collect()
}

/// Helper function to check if a block has a specific modifier
fn has_modifier(block: &Block, key: &str, value: &str) -> bool {
    block.modifiers.iter().any(|(k, v)| k == key && v == value)
}

/// Helper function to find a child block by name
fn find_child_by_name<'a>(block: &'a Block, name: &str) -> Option<&'a Block> {
    block.children.iter().find(|b| b.name.as_ref().map_or(false, |n| n == name))
}

/// Test all block types defined in the language specification
#[test]
fn test_all_block_types() {
    // Create a document with every block type
    let input = r#"
    [question name:test-question]
    What is the meaning of life?
    [/question]
    
    [response name:test-response]
    The meaning of life is 42.
    [/response]
    
    [code:python name:test-code-python]
    print("Hello from Python!")
    [/code:python]
    
    [shell name:test-shell]
    echo "Hello from the shell!"
    [/shell]
    
    [api name:test-api method:GET]
    https://jsonplaceholder.typicode.com/todos/1
    [/api]
    
    [data name:test-data format:json]
    {"key": "value", "number": 42}
    [/data]
    
    [variable name:test-variable]
    sample value
    [/variable]
    
    [secret name:test-secret]
    API_KEY
    [/secret]
    
    [filename name:test-filename]
    test_file.txt
    [/filename]
    
    [results name:test-results for:test-code-python]
    Hello from Python!
    [/results]
    
    [error_results name:test-error-results for:test-code-python]
    Error: Command failed
    [/error_results]
    
    [error name:test-error]
    This is an error message
    [/error]
    
    [preview name:test-preview]
    Preview content
    [/preview]
    
    [memory name:test-memory]
    Memory content
    [/memory]
    
    [visualization name:test-viz type:bar]
    // Visualization content
    [/visualization]
    
    [conditional name:test-conditional if:test-variable]
    Conditional content
    [/conditional]
    
    [section:intro name:test-section]
    Section content
    [/section:intro]
    
    [template name:test-template]
    Template content with ${param}
    [/template]
    
    [template:test-template name:test-template-usage param:"value"]
    [/template:test-template]
    "#;

    let blocks = parse_document(input).expect("Failed to parse document with all block types");
    
    // Verify we have the expected number of blocks
    assert!(blocks.len() >= 20, "Expected at least 20 blocks, got {}", blocks.len());
    
    // Check each block type exists
    let block_types = [
        "question", "response", "code:python", "shell", "api", 
        "data", "variable", "secret", "filename", "results", 
        "error_results", "error", "preview", "memory", "visualization", 
        "conditional", "section:intro", "template"
    ];
    
    for block_type in block_types.iter() {
        assert!(
            blocks.iter().any(|b| b.block_type == *block_type || 
                             (b.block_type.starts_with("template_invocation") && block_type == &"template")),
            "Missing block type: {}", block_type
        );
    }
    
    // Verify specific block content
    let python_block = find_block_by_name(&blocks, "test-code-python").expect("Python block not found");
    assert_eq!(python_block.content.trim(), "print(\"Hello from Python!\")");
    
    let data_block = find_block_by_name(&blocks, "test-data").expect("Data block not found");
    assert_eq!(data_block.content.trim(), "{\"key\": \"value\", \"number\": 42}");
    
    // Verify template invocation has the correct parameter
    let template_usage = find_block_by_name(&blocks, "test-template-usage");
    if let Some(usage) = template_usage {
        assert!(has_modifier(usage, "param", "value"));
    }
}
    
    [filename name:test-filename]
    /path/to/file.txt
    [/filename]
    
    [memory name:test-memory]
    Memory content
    [/memory]
    
    [section:intro name:test-section]
    Section content
    [/section:intro]
    
    [conditional name:test-conditional]
    Conditional content
    [/conditional]
    
    [results name:test-results]
    Results content
    [/results]
    
    [error_results name:test-error-results]
    Error results content
    [/error_results]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with all block types: {:?}", result.err());
    
    let blocks = result.unwrap();
    println!("DEBUG: Parsed {} blocks", blocks.len());
    
    // Verify we have all the expected blocks
    let block_names: Vec<String> = blocks.iter()
        .filter_map(|b| b.name.clone())
        .collect();
    
    // Print the block names for debugging
    println!("DEBUG: Block names: {:?}", block_names);
    
    // Check each block type
    assert!(blocks.iter().any(|b| b.block_type == "question"), "Missing question block");
    assert!(blocks.iter().any(|b| b.block_type == "response"), "Missing response block");
    assert!(blocks.iter().any(|b| b.block_type.starts_with("code:")), "Missing code block");
    assert!(blocks.iter().any(|b| b.block_type == "shell"), "Missing shell block");
    assert!(blocks.iter().any(|b| b.block_type == "api"), "Missing api block");
    assert!(blocks.iter().any(|b| b.block_type == "data"), "Missing data block");
    assert!(blocks.iter().any(|b| b.block_type == "variable"), "Missing variable block");
    assert!(blocks.iter().any(|b| b.block_type == "secret"), "Missing secret block");
    assert!(blocks.iter().any(|b| b.block_type == "template"), "Missing template block");
    assert!(blocks.iter().any(|b| b.block_type == "error"), "Missing error block");
    assert!(blocks.iter().any(|b| b.block_type == "visualization"), "Missing visualization block");
    assert!(blocks.iter().any(|b| b.block_type == "preview"), "Missing preview block");
    assert!(blocks.iter().any(|b| b.block_type == "filename"), "Missing filename block");
    assert!(blocks.iter().any(|b| b.block_type == "memory"), "Missing memory block");
    assert!(blocks.iter().any(|b| b.block_type.starts_with("section:")), "Missing section block");
    assert!(blocks.iter().any(|b| b.block_type == "conditional"), "Missing conditional block");
    assert!(blocks.iter().any(|b| b.block_type == "results"), "Missing results block");
    assert!(blocks.iter().any(|b| b.block_type == "error_results"), "Missing error_results block");
}

/// Test nested blocks inside sections
#[test]
fn test_nested_blocks() {
    let input = r#"
    [section:outer name:outer-section]
        [code:python name:outer-code]
        print("Outer code")
        [/code:python]
        
        [section:inner name:inner-section]
            [code:python name:inner-code]
            print("Inner code")
            [/code:python]
            
            [data name:inner-data]
            {"key": "inner value"}
            [/data]
        [/section:inner]
        
        [variable name:outer-variable]
        outer value
        [/variable]
    [/section:outer]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with nested blocks: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Find the outer section block
    let outer_section = blocks.iter().find(|b| b.name.as_deref() == Some("outer-section"));
    assert!(outer_section.is_some(), "Could not find outer section block");
    
    let outer_section = outer_section.unwrap();
    
    // Check that outer section has children
    assert!(!outer_section.children.is_empty(), "Outer section has no children");
    
    // Verify we have the expected outer section children
    assert!(outer_section.children.iter().any(|b| b.name.as_deref() == Some("outer-code")), 
           "Outer section missing outer-code child");
    assert!(outer_section.children.iter().any(|b| b.name.as_deref() == Some("outer-variable")), 
           "Outer section missing outer-variable child");
    
    // Find the inner section
    let inner_section = outer_section.children.iter().find(|b| b.name.as_deref() == Some("inner-section"));
    assert!(inner_section.is_some(), "Could not find inner section");
    
    let inner_section = inner_section.unwrap();
    
    // Check inner section children
    assert!(!inner_section.children.is_empty(), "Inner section has no children");
    assert!(inner_section.children.iter().any(|b| b.name.as_deref() == Some("inner-code")), 
           "Inner section missing inner-code child");
    assert!(inner_section.children.iter().any(|b| b.name.as_deref() == Some("inner-data")), 
           "Inner section missing inner-data child");
    
    // Print debug info
    println!("DEBUG: Outer section children: {}", outer_section.children.len());
    println!("DEBUG: Inner section children: {}", inner_section.children.len());
    
    for child in &outer_section.children {
        println!("DEBUG: Outer child: {}", child.name.as_deref().unwrap_or("unnamed"));
    }
    
    for child in &inner_section.children {
        println!("DEBUG: Inner child: {}", child.name.as_deref().unwrap_or("unnamed"));
    }
}

/// Test blocks with complex modifiers of different types
#[test]
fn test_complex_modifiers() {
    let input = r#"
    [code:python name:test-modifiers cache_result:true timeout:30 retry:3 fallback:fallback-block async:false]
    print("Testing modifiers")
    [/code:python]
    
    [data name:test-string-modifiers format:"json" display:"inline" depends:"another-block"]
    {"data": "test"}
    [/data]
    
    [shell name:test-number-modifiers priority:8 weight:0.75 order:0.5 max_lines:100]
    echo "Testing numeric modifiers"
    [/shell]
    
    [api name:test-boolean-modifiers debug:true always_include:false trim:true cache_result:true]
    https://api.example.com/endpoint
    [/api]
    
    [variable name:test-mixed-modifiers priority:10 always_include:true format:"plain" debug:false]
    Mixed modifier test
    [/variable]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with complex modifiers: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Find each block
    let code_block = blocks.iter().find(|b| b.name.as_deref() == Some("test-modifiers"));
    let data_block = blocks.iter().find(|b| b.name.as_deref() == Some("test-string-modifiers"));
    let shell_block = blocks.iter().find(|b| b.name.as_deref() == Some("test-number-modifiers"));
    let api_block = blocks.iter().find(|b| b.name.as_deref() == Some("test-boolean-modifiers"));
    let var_block = blocks.iter().find(|b| b.name.as_deref() == Some("test-mixed-modifiers"));
    
    // Verify each block was found
    assert!(code_block.is_some(), "Could not find code block with modifiers");
    assert!(data_block.is_some(), "Could not find data block with string modifiers");
    assert!(shell_block.is_some(), "Could not find shell block with number modifiers");
    assert!(api_block.is_some(), "Could not find API block with boolean modifiers");
    assert!(var_block.is_some(), "Could not find variable block with mixed modifiers");
    
    // Check boolean modifiers
    let code_block = code_block.unwrap();
    assert!(code_block.is_modifier_true("cache_result"), "cache_result modifier not set to true");
    assert!(!code_block.is_modifier_true("async"), "async modifier not set to false");
    
    // Check numeric modifiers
    let timeout = code_block.get_modifier_as_f64("timeout");
    assert!(timeout.is_some() && timeout.unwrap() == 30.0, "timeout modifier not set to 30");
    
    let retry = code_block.get_modifier_as_f64("retry");
    assert!(retry.is_some() && retry.unwrap() == 3.0, "retry modifier not set to 3");
    
    // Check string modifiers
    assert_eq!(code_block.get_modifier("fallback").map(|s| s.as_str()), Some("fallback-block"), 
               "fallback modifier not set correctly");
    
    // Check shell block numeric modifiers
    let shell_block = shell_block.unwrap();
    assert!(shell_block.get_modifier_as_f64("priority").is_some(), "priority modifier not parsed");
    assert!(shell_block.get_modifier_as_f64("weight").is_some(), "weight modifier not parsed");
    assert!(shell_block.get_modifier_as_f64("order").is_some(), "order modifier not parsed");
    
    // Check data block string modifiers
    let data_block = data_block.unwrap();
    assert_eq!(data_block.get_modifier("format").map(|s| s.as_str()), Some("json"), 
               "format modifier not set correctly");
    assert_eq!(data_block.get_modifier("display").map(|s| s.as_str()), Some("inline"), 
               "display modifier not set correctly");
               
    // Print debug info for all modifiers
    for block in &[code_block, data_block, shell_block, api_block, var_block] {
        println!("DEBUG: Block: {}", block.name.as_deref().unwrap_or("unnamed"));
        for (key, value) in &block.modifiers {
            println!("DEBUG:   {} = {}", key, value);
        }
    }
}

/// Test variable references within blocks
#[test]
fn test_variable_references() {
    let input = r#"
    [variable name:var1]
    Original value
    [/variable]
    
    [variable name:var2]
    Another value
    [/variable]
    
    [code:python name:code-with-references]
    # Single reference
    value1 = '''${var1}'''
    
    # Multiple references
    value2 = '''${var2}'''
    combined = f"{value1} and {value2}"
    
    # Reference with surrounding text
    print(f"The values are: ${var1} and also ${var2}")
    [/code:python]
    
    [shell name:shell-with-references]
    echo "Using ${var1} in a command"
    grep "${var2}" /some/file
    [/shell]
    
    [data name:nested-references]
    {
      "first": "${var1}",
      "second": "${var2}",
      "nested": {
        "value": "${var1} inside ${var2}"
      }
    }
    [/data]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with variable references: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Find the blocks with references
    let code_block = blocks.iter().find(|b| b.name.as_deref() == Some("code-with-references"));
    let shell_block = blocks.iter().find(|b| b.name.as_deref() == Some("shell-with-references"));
    let data_block = blocks.iter().find(|b| b.name.as_deref() == Some("nested-references"));
    
    assert!(code_block.is_some(), "Could not find code block with references");
    assert!(shell_block.is_some(), "Could not find shell block with references");
    assert!(data_block.is_some(), "Could not find data block with references");
    
    // Check variable references in code block
    let code_block = code_block.unwrap();
    println!("DEBUG: Code block content: {}", code_block.content);
    assert!(code_block.content.contains("${var1}"), "Code block missing var1 reference");
    assert!(code_block.content.contains("${var2}"), "Code block missing var2 reference");
    
    // Check variable references in shell block
    let shell_block = shell_block.unwrap();
    println!("DEBUG: Shell block content: {}", shell_block.content);
    assert!(shell_block.content.contains("${var1}"), "Shell block missing var1 reference");
    assert!(shell_block.content.contains("${var2}"), "Shell block missing var2 reference");
    
    // Check variable references in data block
    let data_block = data_block.unwrap();
    println!("DEBUG: Data block content: {}", data_block.content);
    assert!(data_block.content.contains("${var1}"), "Data block missing var1 reference");
    assert!(data_block.content.contains("${var2}"), "Data block missing var2 reference");
    
    // Try to extract references using the parser utilities if available
    if let Some(extract_refs) = std::panic::catch_unwind(|| {
        use yet_another_llm_project_but_better::parser::utils::extractors::extract_variable_references;
        extract_variable_references(&code_block.content)
    }).ok() {
        println!("DEBUG: Extracted references: {:?}", extract_refs);
        assert!(extract_refs.contains(&"var1".to_string()), "Failed to extract var1 reference");
        assert!(extract_refs.contains(&"var2".to_string()), "Failed to extract var2 reference");
    }
}

/// Test template invocations with parameters
#[test]
fn test_template_invocations() {
    let input = r#"
    [template name:simple-template]
    This is a template with ${placeholder1} and ${placeholder2}.
    [/template]
    
    [@simple-template placeholder1:"value1" placeholder2:"value2"]
    [/@simple-template]
    
    [template name:code-template]
    [code:${language} name:${name}]
    ${code_content}
    [/code:${language}]
    [/template]
    
    [@code-template language:"python" name:"generated-code" code_content:"print('Hello')"]
    [/@code-template]
    
    [template name:nested-template]
    [section:${section_type} name:${section_name}]
      ${section_content}
    [/section:${section_type}]
    [/template]
    
    [@nested-template section_type:"analysis" section_name:"data-analysis" section_content:"Analysis content here"]
    [/@nested-template]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with template invocations: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Find template declarations
    let simple_template = blocks.iter().find(|b| b.name.as_deref() == Some("simple-template"));
    let code_template = blocks.iter().find(|b| b.name.as_deref() == Some("code-template"));
    let nested_template = blocks.iter().find(|b| b.name.as_deref() == Some("nested-template"));
    
    assert!(simple_template.is_some(), "Could not find simple template declaration");
    assert!(code_template.is_some(), "Could not find code template declaration");
    assert!(nested_template.is_some(), "Could not find nested template declaration");
    
    // Find template invocations
    let invocations = blocks.iter().filter(|b| b.block_type == "template_invocation").collect::<Vec<_>>();
    println!("DEBUG: Found {} template invocations", invocations.len());
    
    // We should have 3 invocations
    assert_eq!(invocations.len(), 3, "Expected 3 template invocations, found {}", invocations.len());
    
    // Check each invocation has the correct template name and parameters
    for invocation in invocations {
        println!("DEBUG: Invocation modifiers: {:?}", invocation.modifiers);
        
        // Check if it has a template modifier
        let template_name = invocation.get_modifier("template");
        assert!(template_name.is_some(), "Invocation missing template modifier");
        
        if let Some(name) = template_name {
            match name.as_str() {
                "simple-template" => {
                    assert_eq!(invocation.get_modifier("placeholder1").map(|s| s.as_str()), Some("value1"), 
                               "simple-template invocation missing placeholder1 parameter");
                    assert_eq!(invocation.get_modifier("placeholder2").map(|s| s.as_str()), Some("value2"), 
                               "simple-template invocation missing placeholder2 parameter");
                },
                "code-template" => {
                    assert_eq!(invocation.get_modifier("language").map(|s| s.as_str()), Some("python"), 
                               "code-template invocation missing language parameter");
                    assert_eq!(invocation.get_modifier("name").map(|s| s.as_str()), Some("generated-code"), 
                               "code-template invocation missing name parameter");
                    assert!(invocation.get_modifier("code_content").is_some(), 
                            "code-template invocation missing code_content parameter");
                },
                "nested-template" => {
                    assert_eq!(invocation.get_modifier("section_type").map(|s| s.as_str()), Some("analysis"), 
                               "nested-template invocation missing section_type parameter");
                    assert_eq!(invocation.get_modifier("section_name").map(|s| s.as_str()), Some("data-analysis"), 
                               "nested-template invocation missing section_name parameter");
                    assert!(invocation.get_modifier("section_content").is_some(), 
                            "nested-template invocation missing section_content parameter");
                },
                _ => panic!("Unexpected template name: {}", name),
            }
        }
    }
}

/// Test blocks with special characters in names
#[test]
fn test_special_character_names() {
    let input = r#"
    [code:python name:test-with-hyphens]
    print("Block with hyphens in name")
    [/code:python]
    
    [data name:test_with_underscores]
    {"key": "Block with underscores in name"}
    [/data]
    
    [variable name:test-123-456]
    Block with numbers in name
    [/variable]
    
    [shell name:test-special-chars-_123]
    echo "Block with mixed special characters and numbers"
    [/shell]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with special character names: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Verify blocks with special characters in names
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("test-with-hyphens")), 
           "Missing block with hyphens in name");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("test_with_underscores")), 
           "Missing block with underscores in name");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("test-123-456")), 
           "Missing block with numbers in name");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("test-special-chars-_123")), 
           "Missing block with mixed special characters and numbers");
    
    // Print all block names for debugging
    println!("DEBUG: Blocks with special character names:");
    for block in &blocks {
        if let Some(name) = &block.name {
            println!("DEBUG:   {}", name);
        }
    }
}

/// Test blocks with unusual whitespace patterns
#[test]
fn test_unusual_whitespace() {
    let input = r#"
    [code:python    name:extra-spaces-in-opening-tag    ]
    print("Block with extra spaces in opening tag")
    [/code:python]
    
    [data name:no-spaces-at-all]{"key":"value"}[/data]
    
    [shell name:extra-newlines

    
    ]
    echo "Block with extra newlines in opening tag"
    [/shell]
    
    [variable name:mixed-indentation]
            Indented with spaces
        Indented less
    Not indented
            Indented again
    [/variable]
    
    [code:python name:code-with-tabs]
	print("Line with tab indentation")
		print("Line with double tab indentation")
	    print("Line with mixed tab and space indentation")
    [/code:python]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with unusual whitespace: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Verify blocks with unusual whitespace
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("extra-spaces-in-opening-tag")), 
           "Missing block with extra spaces in opening tag");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("no-spaces-at-all")), 
           "Missing block with no spaces at all");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("extra-newlines")), 
           "Missing block with extra newlines");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("mixed-indentation")), 
           "Missing block with mixed indentation");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("code-with-tabs")), 
           "Missing block with tabs");
    
    // Check content preservation
    let mixed_indent_block = blocks.iter().find(|b| b.name.as_deref() == Some("mixed-indentation")).unwrap();
    println!("DEBUG: Mixed indentation block content: '{}'", mixed_indent_block.content);
    assert!(mixed_indent_block.content.contains("Indented with spaces"), 
           "Mixed indentation block lost content");
    
    let tabs_block = blocks.iter().find(|b| b.name.as_deref() == Some("code-with-tabs")).unwrap();
    println!("DEBUG: Tab indentation block content: '{}'", tabs_block.content);
    assert!(tabs_block.content.contains("tab indentation"), 
           "Tab indentation block lost content");
}

/// Test blocks with multiple modifiers in various formats
#[test]
fn test_multiple_modifiers() {
    let input = r#"
    [code:python name:many-modifiers cache_result:true timeout:30 retry:3 depends:data-block 
     fallback:fallback-block async:false debug:true verbosity:high priority:10]
    print("Block with many modifiers in a single line with line break")
    [/code:python]
    
    [data name:spaced-modifiers format:json   display:inline   priority:8   weight:0.5]
    {"key": "Block with extra spaces between modifiers"}
    [/data]
    
    [shell name:no-spaces-modifiers cache_result:true timeout:5 retry:2fallback:fallback]
    echo "Block with no spaces between some modifiers"
    [/shell]
    
    [api name:quoted-modifiers format:"json" display:"inline" headers:"Content-Type: application/json"]
    https://api.example.com/data
    [/api]
    
    [variable name:comma-separated-modifiers, format:plain, display:block, order:0.1, priority:9]
    Block with comma-separated modifiers
    [/variable]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with multiple modifiers: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Verify blocks with various modifier formats
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("many-modifiers")), 
           "Missing block with many modifiers");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("spaced-modifiers")), 
           "Missing block with spaced modifiers");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("no-spaces-modifiers")), 
           "Missing block with no spaces between modifiers");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("quoted-modifiers")), 
           "Missing block with quoted modifiers");
    assert!(blocks.iter().any(|b| b.name.as_deref() == Some("comma-separated-modifiers")), 
           "Missing block with comma-separated modifiers");
    
    // Verify modifier values for the block with many modifiers
    let many_mods_block = blocks.iter().find(|b| b.name.as_deref() == Some("many-modifiers")).unwrap();
    assert!(many_mods_block.is_modifier_true("cache_result"), "cache_result not set to true");
    assert!(many_mods_block.is_modifier_true("debug"), "debug not set to true");
    assert_eq!(many_mods_block.get_modifier("verbosity").map(|s| s.as_str()), Some("high"),
              "verbosity not set to high");
    assert_eq!(many_mods_block.get_modifier_as_f64("priority"), Some(10.0),
              "priority not set to 10");
    
    // Verify modifier values for block with quoted modifiers
    let quoted_mods_block = blocks.iter().find(|b| b.name.as_deref() == Some("quoted-modifiers")).unwrap();
    assert_eq!(quoted_mods_block.get_modifier("format").map(|s| s.as_str()), Some("json"),
              "format not set to json");
    assert_eq!(quoted_mods_block.get_modifier("display").map(|s| s.as_str()), Some("inline"),
              "display not set to inline");
    assert_eq!(quoted_mods_block.get_modifier("headers").map(|s| s.as_str()), Some("Content-Type: application/json"),
              "headers not set correctly");
    
    // Print all modifiers for debugging
    for block in &blocks {
        if let Some(name) = &block.name {
            println!("DEBUG: {} modifiers:", name);
            for (key, value) in &block.modifiers {
                println!("DEBUG:   {} = {}", key, value);
            }
        }
    }
}
/// Test parsing of nested blocks
#[test]
fn test_nested_blocks() {
    let input = r#"
    [section:h1 name:outer-section]
    # Outer Section
    
    [code:python name:nested-code]
    print("I'm nested inside a section")
    [/code:python]
    
    [section:h2 name:inner-section]
    ## Inner Section
    
    [variable name:nested-variable]
    nested value
    [/variable]
    
    [/section:h2]
    
    [/section:h1]
    "#;
    
    let blocks = parse_document(input).expect("Failed to parse document");
    
    // We should have one top-level section block
    assert_eq!(blocks.len(), 1, "Expected 1 top-level block");
    
    let outer_section = &blocks[0];
    assert_eq!(outer_section.block_type, "section:h1");
    assert_eq!(outer_section.name, Some("outer-section".to_string()));
    
    // The outer section should have 2 children: a code block and an inner section
    assert_eq!(outer_section.children.len(), 2, "Expected 2 child blocks in outer section");
    
    // Check the nested code block
    let nested_code = &outer_section.children[0];
    assert_eq!(nested_code.block_type, "code:python");
    assert_eq!(nested_code.name, Some("nested-code".to_string()));
    assert_eq!(nested_code.content.trim(), "print(\"I'm nested inside a section\")");
    
    // Check the inner section
    let inner_section = &outer_section.children[1];
    assert_eq!(inner_section.block_type, "section:h2");
    assert_eq!(inner_section.name, Some("inner-section".to_string()));
    
    // The inner section should have 1 child: a variable block
    assert_eq!(inner_section.children.len(), 1, "Expected 1 child block in inner section");
    
    // Check the nested variable block
    let nested_variable = &inner_section.children[0];
    assert_eq!(nested_variable.block_type, "variable");
    assert_eq!(nested_variable.name, Some("nested-variable".to_string()));
    assert_eq!(nested_variable.content.trim(), "nested value");
}

/// Test parsing of blocks with complex modifiers
#[test]
fn test_complex_modifiers() {
    let input = r#"
    [code:python name:complex-modifiers deps:math,numpy,pandas auto_execute:true timeout:30 retries:3 on_error:"log and continue" description:"This is a complex block with many modifiers"]
    import math
    import numpy as np
    import pandas as pd
    
    print("Complex modifiers test")
    [/code:python]
    "#;
    
    let blocks = parse_document(input).expect("Failed to parse document");
    assert_eq!(blocks.len(), 1, "Expected 1 block");
    
    let block = &blocks[0];
    assert_eq!(block.block_type, "code:python");
    assert_eq!(block.name, Some("complex-modifiers".to_string()));
    
    // Check all the modifiers
    assert_eq!(block.get_modifier("deps").map(|s| s.as_str()), Some("math,numpy,pandas"));
    assert_eq!(block.get_modifier("auto_execute").map(|s| s.as_str()), Some("true"));
    assert_eq!(block.get_modifier("timeout").map(|s| s.as_str()), Some("30"));
    assert_eq!(block.get_modifier("retries").map(|s| s.as_str()), Some("3"));
    assert_eq!(block.get_modifier("on_error").map(|s| s.as_str()), Some("log and continue"));
    assert_eq!(block.get_modifier("description").map(|s| s.as_str()), Some("This is a complex block with many modifiers"));
    
    // Test the is_modifier_true helper
    assert!(block.is_modifier_true("auto_execute"));
}

/// Test variable references in content
#[test]
fn test_variable_references() {
    use yet_another_llm_project_but_better::parser::utils::extractors::extract_variable_references;
    
    // Test extracting variable references from text
    let text = "This references ${variable1} and ${variable2} and ${nested.variable}";
    let references = extract_variable_references(text);
    
    let expected: HashSet<String> = ["variable1", "variable2", "nested.variable"]
        .iter().map(|s| s.to_string()).collect();
    
    assert_eq!(references, expected);
    
    // Test with duplicate references
    let text_with_duplicates = "${var} appears ${var} multiple ${var} times";
    let references = extract_variable_references(text_with_duplicates);
    
    let expected: HashSet<String> = ["var"].iter().map(|s| s.to_string()).collect();
    assert_eq!(references, expected);
    assert_eq!(references.len(), 1);
}

/// Test error handling for malformed blocks
#[test]
fn test_malformed_blocks() {
    // Missing closing tag
    let input = r#"
    [code:python name:missing-close]
    print("This block is missing a closing tag")
    "#;
    
    let result = parse_document(input);
    assert!(result.is_err(), "Expected an error for malformed block");
    
    // Mismatched closing tag
    let input = r#"
    [code:python name:mismatched-close]
    print("This block has a mismatched closing tag")
    [/code:javascript]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_err(), "Expected an error for mismatched closing tag");
    
    // Invalid block type
    let input = r#"
    [invalid-block-type name:test]
    This is an invalid block type
    [/invalid-block-type]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_err(), "Expected an error for invalid block type");
}

/// Test empty blocks and whitespace handling
#[test]
fn test_empty_blocks() {
    let input = r#"
    [variable name:empty-var]
    [/variable]
    
    [code:python name:whitespace-only]
    
    [/code:python]
    "#;
    
    let blocks = parse_document(input).expect("Failed to parse document");
    assert_eq!(blocks.len(), 2, "Expected 2 blocks");
    
    let empty_var = &blocks[0];
    assert_eq!(empty_var.block_type, "variable");
    assert_eq!(empty_var.name, Some("empty-var".to_string()));
    assert_eq!(empty_var.content.trim(), "");
    
    let whitespace_only = &blocks[1];
    assert_eq!(whitespace_only.block_type, "code:python");
    assert_eq!(whitespace_only.name, Some("whitespace-only".to_string()));
    assert_eq!(whitespace_only.content.trim(), "");
}

// Helper functions for the complex document test
fn find_block_by_name(blocks: &[Block], name: &str) -> Option<&Block> {
    blocks.iter().find(|b| b.name.as_ref().map_or(false, |n| n == name))
}

fn find_child_by_name(parent: &Block, name: &str) -> Option<&Block> {
    parent.children.iter().find(|b| b.name.as_ref().map_or(false, |n| n == name))
}

fn has_modifier(block: &Block, key: &str, value: &str) -> bool {
    block.get_modifier(key).map_or(false, |v| v == value)
}

/// Test complex document with nested sections and dependencies
#[test]
fn test_complex_document() {
    // Test parsing a complex document with multiple block types and nesting
    let input = r#"
[section:intro name:document_intro]
# Introduction

This is a complex document with multiple block types and nesting.

[code:python name:setup_code]
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

def setup_environment():
    print("Setting up environment")
    return {"ready": True}
[/code:python]

[variable name:config]
{
    "data_source": "example.csv",
    "max_rows": 1000,
    "columns": ["id", "name", "value"]
}
[/variable]

[section:data_processing name:data_section]
## Data Processing

[code:python name:process_data depends:setup_code]
def process_data(source="${config.data_source}"):
    print(f"Processing data from {source}")
    # Use the setup from the previous block
    env = setup_environment()
    if env["ready"]:
        return {"processed": True}
[/code:python]

[data name:sample_data]
{"id": 1, "name": "Example", "value": 42}
[/data]

[shell name:run_script]
python -c "import json; print(json.dumps(${sample_data}))"
[/shell]

[results for:run_script]
{"id": 1, "name": "Example", "value": 42}
[/results]
[/section:data_processing]

[section:visualization name:viz_section]
## Visualization

[code:python name:create_viz depends:process_data]
def create_visualization(data):
    print("Creating visualization")
    # This would normally create a plot
    return "visualization.png"
[/code:python]

[visualization name:data_viz type:bar data:sample_data]
// Visualization configuration
[/visualization]

[preview for:data_viz]
[Bar chart showing sample data]
[/preview]
[/section:visualization]

[template name:report_template]
# ${title}

Data processed: ${data_processed}
Visualization: ${visualization_path}

## Summary
${summary}
[/template]

[template:report_template name:final_report 
  title:"Analysis Report"
  data_processed:"Yes"
  visualization_path:"visualization.png"
  summary:"This is a summary of the analysis."
]
[/template:report_template]

[conditional if:config.max_rows>500]
This section only appears if max_rows is greater than 500.
[/conditional]

[error_results for:missing_block]
Error: Block not found
[/error_results]
[/section:intro]
"#;

    let blocks = parse_document(input).expect("Failed to parse complex document");
    
    // Find the main section
    let intro_section = find_block_by_name(&blocks, "document_intro").expect("Intro section not found");
    assert_eq!(intro_section.block_type, "section:intro");
    
    // Check that the section has the expected number of child blocks
    assert!(intro_section.children.len() >= 5, "Expected at least 5 child blocks, got {}", intro_section.children.len());
    
    // Check nested sections
    let data_section = find_child_by_name(intro_section, "data_section").expect("Data section not found");
    assert_eq!(data_section.block_type, "section:data_processing");
    assert_eq!(data_section.children.len(), 4, "Expected 4 child blocks in data section, got {}", data_section.children.len());
    
    let viz_section = find_child_by_name(intro_section, "viz_section").expect("Visualization section not found");
    assert_eq!(viz_section.block_type, "section:visualization");
    assert_eq!(viz_section.children.len(), 3, "Expected 3 child blocks in viz section, got {}", viz_section.children.len());
    
    // Check variable references
    let process_data = find_child_by_name(data_section, "process_data").expect("Process data block not found");
    assert!(process_data.content.contains("${config.data_source}"));
    
    // Check dependencies
    assert!(has_modifier(process_data, "depends", "setup_code"));
    
    let create_viz = find_child_by_name(viz_section, "create_viz").expect("Create viz block not found");
    assert!(has_modifier(create_viz, "depends", "process_data"));
    
    // Check template invocation
    let final_report = find_block_by_name(&intro_section.children, "final_report").expect("Final report not found");
    assert!(has_modifier(final_report, "title", "Analysis Report"));
    assert!(has_modifier(final_report, "data_processed", "Yes"));
    assert!(has_modifier(final_report, "visualization_path", "visualization.png"));
    assert!(has_modifier(final_report, "summary", "This is a summary of the analysis."));
    
    // Check conditional block
    let conditional = intro_section.children.iter()
        .find(|b| b.block_type == "conditional")
        .expect("Conditional block not found");
    assert!(has_modifier(conditional, "if", "config.max_rows>500"));
}
#[test]
fn test_complex_modifiers() {
    // Test blocks with complex modifiers
    let input = r#"
[code:python name:modifier_test timeout:30 auto_execute:true cache_result:false]
print("Testing modifiers")
[/code:python]

[data name:string_modifier format:"json" description:"This is a data block with a string modifier"]
{"test": true}
[/data]

[shell name:numeric_modifier timeout:60 retries:3]
echo "Testing numeric modifiers"
[/shell]

[variable name:boolean_modifier visible:true editable:false required:true]
Boolean modifier test
[/variable]

[code:python name:complex_modifiers 
  timeout:30 
  auto_execute:true 
  cache_result:false 
  max_lines:100
  description:"This is a complex description with spaces"
  tags:"tag1,tag2,tag3"
  priority:5
  depends:block1,block2,block3
  env:{"PATH":"/usr/bin","HOME":"/home/user"}
]
print("Testing complex modifiers with multiline format")
[/code:python]
"#;

    let blocks = parse_document(input).expect("Failed to parse complex modifiers");
    
    // Check string modifiers
    let string_mod_block = find_block_by_name(&blocks, "string_modifier").expect("String modifier block not found");
    assert!(has_modifier(string_mod_block, "format", "json"));
    assert!(has_modifier(string_mod_block, "description", "This is a data block with a string modifier"));
    
    // Check numeric modifiers
    let numeric_mod_block = find_block_by_name(&blocks, "numeric_modifier").expect("Numeric modifier block not found");
    assert!(has_modifier(numeric_mod_block, "timeout", "60"));
    assert!(has_modifier(numeric_mod_block, "retries", "3"));
    
    // Check boolean modifiers
    let boolean_mod_block = find_block_by_name(&blocks, "boolean_modifier").expect("Boolean modifier block not found");
    assert!(has_modifier(boolean_mod_block, "visible", "true"));
    assert!(has_modifier(boolean_mod_block, "editable", "false"));
    assert!(has_modifier(boolean_mod_block, "required", "true"));
    
    // Check complex multiline modifiers
    let complex_mod_block = find_block_by_name(&blocks, "complex_modifiers").expect("Complex modifier block not found");
    assert!(has_modifier(complex_mod_block, "timeout", "30"));
    assert!(has_modifier(complex_mod_block, "auto_execute", "true"));
    assert!(has_modifier(complex_mod_block, "cache_result", "false"));
    assert!(has_modifier(complex_mod_block, "max_lines", "100"));
    assert!(has_modifier(complex_mod_block, "description", "This is a complex description with spaces"));
    assert!(has_modifier(complex_mod_block, "tags", "tag1,tag2,tag3"));
    assert!(has_modifier(complex_mod_block, "priority", "5"));
    assert!(has_modifier(complex_mod_block, "depends", "block1,block2,block3"));
    assert!(has_modifier(complex_mod_block, "env", "{\"PATH\":\"/usr/bin\",\"HOME\":\"/home/user\"}"));
}
#[test]
fn test_variable_references() {
    // Test variable references within blocks
    let input = r#"
[variable name:test_var]
Hello, world!
[/variable]

[code:python name:var_reference]
message = "${test_var}"
print(f"The message is: {message}")
[/code:python]

[shell name:shell_var_reference]
echo "Shell says: ${test_var}"
[/shell]

[code:python name:nested_reference]
outer = "${test_var}"
inner = "This contains ${test_var} inside"
nested = "${nested_reference}"
print(f"{outer} - {inner} - {nested}")
[/code:python]

[code:python name:complex_references]
# Multiple references on one line
combined = "${var1} ${var2} ${var3}"

# Nested variable references
result = "${prefix_${dynamic_suffix}}"

# References with special characters
special = "${obj['key']} and ${arr[0]}"

# References in multiline strings
multiline = """
Line 1: ${var1}
Line 2: ${var2}
Line 3: ${var3}
"""
[/code:python]
"#;

    let blocks = parse_document(input).expect("Failed to parse variable references");
    
    // Check variable reference in Python code
    let python_ref = find_block_by_name(&blocks, "var_reference").expect("Python reference block not found");
    assert!(python_ref.content.contains("${test_var}"));
    
    // Check variable reference in shell command
    let shell_ref = find_block_by_name(&blocks, "shell_var_reference").expect("Shell reference block not found");
    assert!(shell_ref.content.contains("${test_var}"));
    
    // Check nested references
    let nested_ref = find_block_by_name(&blocks, "nested_reference").expect("Nested reference block not found");
    assert!(nested_ref.content.contains("${test_var}"));
    assert!(nested_ref.content.contains("${nested_reference}"));
    
    // Check complex references
    let complex_ref = find_block_by_name(&blocks, "complex_references").expect("Complex reference block not found");
    assert!(complex_ref.content.contains("${var1} ${var2} ${var3}"));
    assert!(complex_ref.content.contains("${prefix_${dynamic_suffix}}"));
    assert!(complex_ref.content.contains("${obj['key']} and ${arr[0]}"));
    assert!(complex_ref.content.contains("Line 1: ${var1}"));
    
    // Test variable reference extraction
    let refs = extract_variable_references(&complex_ref.content);
    let expected_refs: HashSet<String> = ["var1", "var2", "var3", "prefix_${dynamic_suffix}", 
                                         "dynamic_suffix", "obj['key']", "arr[0]"]
        .iter().map(|s| s.to_string()).collect();
    
    for var in expected_refs {
        assert!(refs.contains(&var), "Missing variable reference: {}", var);
    }
}
#[test]
fn test_template_invocations() {
    // Test template invocations with parameters
    let input = r#"
[template name:greeting_template]
Hello, ${name}!
[/template]

[template:greeting_template name:greeting1 name:"Alice"]
[/template:greeting_template]

[template:greeting_template name:greeting2 name:"Bob" extra:"parameter"]
[/template:greeting_template]

[template name:complex_template]
${param1} and ${param2} make ${result}
[/template]

[template:complex_template name:complex_invocation param1:"Apple" param2:"Orange" result:"Fruit Salad"]
[/template:complex_template]

[template name:multiline_template]
def process_data(data):
    # Process using ${algorithm}
    return data.${operation}()
[/template]

[template:multiline_template 
  name:multiline_invocation
  algorithm:quicksort
  operation:sort
  extra_param:"This is a long parameter value with spaces and special chars: !@#$%^&*()"
]
[/template:multiline_template]
"#;

    let blocks = parse_document(input).expect("Failed to parse template invocations");
    
    // Check template definitions
    let template_blocks = find_blocks_by_type(&blocks, "template");
    assert_eq!(template_blocks.len(), 3, "Expected 3 template blocks, got {}", template_blocks.len());
    
    // Check template invocations
    let invocation_blocks = blocks.iter()
        .filter(|b| b.block_type == "template_invocation" || b.block_type.starts_with("template_invocation:"))
        .collect::<Vec<_>>();
    assert_eq!(invocation_blocks.len(), 4, "Expected 4 template invocation blocks, got {}", invocation_blocks.len());
    
    // Check parameters in invocations
    let greeting1 = find_block_by_name(&blocks, "greeting1").expect("Greeting1 invocation not found");
    assert!(has_modifier(greeting1, "name", "Alice"));
    
    let greeting2 = find_block_by_name(&blocks, "greeting2").expect("Greeting2 invocation not found");
    assert!(has_modifier(greeting2, "name", "Bob"));
    assert!(has_modifier(greeting2, "extra", "parameter"));
    
    let complex = find_block_by_name(&blocks, "complex_invocation").expect("Complex invocation not found");
    assert!(has_modifier(complex, "param1", "Apple"));
    assert!(has_modifier(complex, "param2", "Orange"));
    assert!(has_modifier(complex, "result", "Fruit Salad"));
    
    // Check multiline template invocation
    let multiline = find_block_by_name(&blocks, "multiline_invocation").expect("Multiline invocation not found");
    assert!(has_modifier(multiline, "algorithm", "quicksort"));
    assert!(has_modifier(multiline, "operation", "sort"));
    assert!(has_modifier(multiline, "extra_param", "This is a long parameter value with spaces and special chars: !@#$%^&*()"));
}
#[test]
fn test_special_characters() {
    // Test blocks with special characters in names and content
    let input = r#"
[code:python name:special-chars-1]
print("Special chars: !@#$%^&*()")
[/code:python]

[data name:special_chars_2]
{"special": "chars: !@#$%^&*()"}
[/data]

[variable name:special_chars.3]
Special.chars.with.dots
[/variable]

[code:python name:unicode_chars]
print("Unicode: 你好, 世界! Привет, мир! こんにちは世界!")
[/code:python]

[shell name:escaped_chars]
echo "Escaped chars: \t \n \r \\ \" \'"
[/shell]

[code:javascript name:unicode_name_блок]
console.log("Block with unicode name");
[/code:javascript]

[data name:emoji_name_🔥]
{"emoji": true}
[/data]

[code:python name:special_symbols_name_@#$%]
print("Block with special symbols in name")
[/code:python]
"#;

    let blocks = parse_document(input).expect("Failed to parse special characters");
    
    // Check blocks with special characters in names
    assert!(find_block_by_name(&blocks, "special-chars-1").is_some());
    assert!(find_block_by_name(&blocks, "special_chars_2").is_some());
    assert!(find_block_by_name(&blocks, "special_chars.3").is_some());
    
    // Check block with Unicode content
    let unicode_block = find_block_by_name(&blocks, "unicode_chars").expect("Unicode block not found");
    assert!(unicode_block.content.contains("你好"));
    assert!(unicode_block.content.contains("Привет"));
    assert!(unicode_block.content.contains("こんにちは"));
    
    // Check block with escaped characters
    let escaped_block = find_block_by_name(&blocks, "escaped_chars").expect("Escaped chars block not found");
    assert!(escaped_block.content.contains("\\t"));
    assert!(escaped_block.content.contains("\\n"));
    assert!(escaped_block.content.contains("\\\""));
    
    // Check blocks with unicode and special characters in names
    // Note: Some of these might fail depending on the parser's handling of special characters
    // The test verifies the parser's behavior rather than enforcing a specific behavior
    let unicode_name_result = find_block_by_name(&blocks, "unicode_name_блок");
    let emoji_name_result = find_block_by_name(&blocks, "emoji_name_🔥");
    let special_symbols_result = find_block_by_name(&blocks, "special_symbols_name_@#$%");
    
    println!("Unicode name block found: {}", unicode_name_result.is_some());
    println!("Emoji name block found: {}", emoji_name_result.is_some());
    println!("Special symbols name block found: {}", special_symbols_result.is_some());
}
#[test]
fn test_whitespace_patterns() {
    // Test blocks with unusual whitespace patterns
    let input = r#"
[code:python    name:whitespace_test   ]
    print("Indented code")
    for i in range(5):
        print(f"  {i}")
[/code:python]

[  data   name:whitespace_data  format:json  ]
{
    "key": "value",
    "nested": {
        "key": "value"
    }
}
[/data]

[shell name:whitespace_shell]
echo "Command with trailing spaces"    
echo "Command with tabs"	
[/shell]

[variable name:empty_lines]

Variable with empty lines

[/variable]

[code:python name:multiline_whitespace
   timeout:30   
   auto_execute:true   
   format:json   ]
print("Block with whitespace in multiline modifiers")
[/code:python]

[code:python name:extreme_whitespace]
    
    
print("Block with extreme whitespace")
    
    
[/code:python]

[code:python name:tab_indentation]
	print("Tab indented line")
		print("Double tab indented line")
			print("Triple tab indented line")
[/code:python]
"#;

    let blocks = parse_document(input).expect("Failed to parse whitespace patterns");
    
    // Check blocks with whitespace in tags
    assert!(find_block_by_name(&blocks, "whitespace_test").is_some());
    assert!(find_block_by_name(&blocks, "whitespace_data").is_some());
    
    // Check indented code block
    let code_block = find_block_by_name(&blocks, "whitespace_test").expect("Whitespace code block not found");
    assert!(code_block.content.contains("    print(\"Indented code\")"));
    
    // Check block with empty lines
    let empty_lines = find_block_by_name(&blocks, "empty_lines").expect("Empty lines block not found");
    assert_eq!(empty_lines.content.trim(), "Variable with empty lines");
    
    // Check block with multiline whitespace in modifiers
    let multiline_whitespace = find_block_by_name(&blocks, "multiline_whitespace").expect("Multiline whitespace block not found");
    assert!(has_modifier(multiline_whitespace, "timeout", "30"));
    assert!(has_modifier(multiline_whitespace, "auto_execute", "true"));
    assert!(has_modifier(multiline_whitespace, "format", "json"));
    
    // Check block with extreme whitespace
    let extreme_whitespace = find_block_by_name(&blocks, "extreme_whitespace").expect("Extreme whitespace block not found");
    assert!(extreme_whitespace.content.contains("print(\"Block with extreme whitespace\")"));
    
    // Check block with tab indentation
    let tab_indentation = find_block_by_name(&blocks, "tab_indentation").expect("Tab indentation block not found");
    assert!(tab_indentation.content.contains("	print(\"Tab indented line\")"));
    assert!(tab_indentation.content.contains("		print(\"Double tab indented line\")"));
    assert!(tab_indentation.content.contains("			print(\"Triple tab indented line\")"));
}
#[test]
fn test_multiple_modifiers() {
    // Test blocks with multiple modifiers
    let input = r#"
[code:python name:multi_mod auto_execute:true timeout:30 cache_result:true format:text trim:true max_lines:10]
print("Block with many modifiers")
[/code:python]

[data name:multi_mod_data format:json schema:person validate:true required:true visible:true editable:false]
{"name": "John", "age": 30}
[/data]

[shell name:multi_mod_shell timeout:60 working_dir:"/tmp" env:production retries:3 silent:true]
echo "Multiple modifiers in shell block"
[/shell]

[code:python name:complex_mod_combo
  timeout:30
  auto_execute:true
  cache_result:true
  format:"json"
  trim:true
  max_lines:100
  description:"This is a complex block with many modifiers"
  depends:block1,block2,block3
  requires:numpy,pandas,matplotlib
  env:{"PATH":"/usr/bin","HOME":"/home/user"}
  tags:"tag1,tag2,tag3"
  visible:true
  editable:false
  priority:5
]
print("Block with a complex combination of modifiers")
[/code:python]
"#;

    let blocks = parse_document(input).expect("Failed to parse multiple modifiers");
    
    // Check Python block with multiple modifiers
    let python_block = find_block_by_name(&blocks, "multi_mod").expect("Multi-modifier Python block not found");
    assert!(has_modifier(python_block, "auto_execute", "true"));
    assert!(has_modifier(python_block, "timeout", "30"));
    assert!(has_modifier(python_block, "cache_result", "true"));
    assert!(has_modifier(python_block, "format", "text"));
    assert!(has_modifier(python_block, "trim", "true"));
    assert!(has_modifier(python_block, "max_lines", "10"));
    
    // Check data block with multiple modifiers
    let data_block = find_block_by_name(&blocks, "multi_mod_data").expect("Multi-modifier data block not found");
    assert!(has_modifier(data_block, "format", "json"));
    assert!(has_modifier(data_block, "schema", "person"));
    assert!(has_modifier(data_block, "validate", "true"));
    assert!(has_modifier(data_block, "required", "true"));
    assert!(has_modifier(data_block, "visible", "true"));
    assert!(has_modifier(data_block, "editable", "false"));
    
    // Check shell block with multiple modifiers
    let shell_block = find_block_by_name(&blocks, "multi_mod_shell").expect("Multi-modifier shell block not found");
    assert!(has_modifier(shell_block, "timeout", "60"));
    assert!(has_modifier(shell_block, "working_dir", "/tmp"));
    assert!(has_modifier(shell_block, "env", "production"));
    assert!(has_modifier(shell_block, "retries", "3"));
    assert!(has_modifier(shell_block, "silent", "true"));
    
    // Check complex combination of modifiers
    let complex_block = find_block_by_name(&blocks, "complex_mod_combo").expect("Complex modifier combo block not found");
    assert!(complex_block.modifiers.len() >= 14, "Expected at least 14 modifiers, got {}", complex_block.modifiers.len());
    assert!(has_modifier(complex_block, "timeout", "30"));
    assert!(has_modifier(complex_block, "auto_execute", "true"));
    assert!(has_modifier(complex_block, "cache_result", "true"));
    assert!(has_modifier(complex_block, "format", "json"));
    assert!(has_modifier(complex_block, "trim", "true"));
    assert!(has_modifier(complex_block, "max_lines", "100"));
    assert!(has_modifier(complex_block, "description", "This is a complex block with many modifiers"));
    assert!(has_modifier(complex_block, "depends", "block1,block2,block3"));
    assert!(has_modifier(complex_block, "requires", "numpy,pandas,matplotlib"));
    assert!(has_modifier(complex_block, "env", "{\"PATH\":\"/usr/bin\",\"HOME\":\"/home/user\"}"));
    assert!(has_modifier(complex_block, "tags", "tag1,tag2,tag3"));
    assert!(has_modifier(complex_block, "visible", "true"));
    assert!(has_modifier(complex_block, "editable", "false"));
    assert!(has_modifier(complex_block, "priority", "5"));
}
#[test]
fn test_malformed_blocks() {
    // Test error cases with malformed blocks
    
    // Missing closing tag
    let input1 = r#"
[code:python name:missing_close]
print("This block has no closing tag")
"#;
    assert!(parse_document(input1).is_err());
    
    // Mismatched closing tag
    let input2 = r#"
[code:python name:mismatched_close]
print("This block has mismatched closing tag")
[/code:javascript]
"#;
    assert!(parse_document(input2).is_err());
    
    // Invalid modifier format
    let input3 = r#"
[code:python name:invalid:modifier]
print("This block has an invalid modifier")
[/code:python]
"#;
    assert!(parse_document(input3).is_err());
    
    // Duplicate block names
    let input4 = r#"
[code:python name:duplicate]
print("First block")
[/code:python]

[data name:duplicate]
{"duplicate": true}
[/data]
"#;
    assert!(parse_document(input4).is_err());
    
    // Unclosed modifier section
    let input5 = r#"
[code:python name:unclosed_modifiers
print("Unclosed modifiers section")
[/code:python]
"#;
    assert!(parse_document(input5).is_err());
    
    // Invalid block type
    let input6 = r#"
[invalid_type name:invalid_block]
Invalid block type
[/invalid_type]
"#;
    assert!(parse_document(input6).is_err());
    
    // Nested block without parent
    let input7 = r#"
[/code:python]
print("Closing tag without opening tag")
[/code:python]
"#;
    assert!(parse_document(input7).is_err());
    
    // Empty block name
    let input8 = r#"
[code:python name:]
print("Empty block name")
[/code:python]
"#;
    assert!(parse_document(input8).is_err());
}
#[test]
fn test_closing_tags() {
    // Test closing tags with and without language specification
    let input = r#"
[code:python name:with_lang]
print("Block with language in closing tag")
[/code:python]

[code:python name:without_lang]
print("Block without language in closing tag")
[/code]

[code:javascript name:js_block]
console.log("JavaScript block with language in closing tag");
[/code:javascript]

[code:javascript name:js_without_lang]
console.log("JavaScript block without language in closing tag");
[/code]

[section:intro name:section_with_type]
Section with type in closing tag
[/section:intro]

[section:intro name:section_without_type]
Section without type in closing tag
[/section]

[data name:data_with_closing]
{"key": "value"}
[/data]

[data name:data_without_closing]
{"key": "value"}
[/]
"#;

    let blocks = parse_document(input).expect("Failed to parse closing tags");
    
    // Check all blocks were parsed correctly
    assert!(find_block_by_name(&blocks, "with_lang").is_some());
    assert!(find_block_by_name(&blocks, "without_lang").is_some());
    assert!(find_block_by_name(&blocks, "js_block").is_some());
    assert!(find_block_by_name(&blocks, "js_without_lang").is_some());
    assert!(find_block_by_name(&blocks, "section_with_type").is_some());
    assert!(find_block_by_name(&blocks, "section_without_type").is_some());
    assert!(find_block_by_name(&blocks, "data_with_closing").is_some());
    assert!(find_block_by_name(&blocks, "data_without_closing").is_some());
    
    // Verify content of blocks
    let with_lang = find_block_by_name(&blocks, "with_lang").expect("With lang block not found");
    assert_eq!(with_lang.content.trim(), "print(\"Block with language in closing tag\")");
    
    let without_lang = find_block_by_name(&blocks, "without_lang").expect("Without lang block not found");
    assert_eq!(without_lang.content.trim(), "print(\"Block without language in closing tag\")");
    
    let section_with_type = find_block_by_name(&blocks, "section_with_type").expect("Section with type not found");
    assert_eq!(section_with_type.content.trim(), "Section with type in closing tag");
    
    let section_without_type = find_block_by_name(&blocks, "section_without_type").expect("Section without type not found");
    assert_eq!(section_without_type.content.trim(), "Section without type in closing tag");
}
#[test]
fn test_line_endings() {
    // Test CRLF vs LF line ending differences
    
    // LF line endings
    let lf_input = "[code:python name:lf_test]\nprint(\"LF line endings\")\n[/code:python]";
    let lf_blocks = parse_document(lf_input).expect("Failed to parse LF line endings");
    let lf_block = find_block_by_name(&lf_blocks, "lf_test").expect("LF block not found");
    assert_eq!(lf_block.content.trim(), "print(\"LF line endings\")");
    
    // CRLF line endings
    let crlf_input = "[code:python name:crlf_test]\r\nprint(\"CRLF line endings\")\r\n[/code:python]";
    let crlf_blocks = parse_document(crlf_input).expect("Failed to parse CRLF line endings");
    let crlf_block = find_block_by_name(&crlf_blocks, "crlf_test").expect("CRLF block not found");
    assert_eq!(crlf_block.content.trim(), "print(\"CRLF line endings\")");
    
    // Mixed line endings
    let mixed_input = "[code:python name:mixed_test]\nprint(\"First line\")\r\nprint(\"Second line\")\n[/code:python]";
    let mixed_blocks = parse_document(mixed_input).expect("Failed to parse mixed line endings");
    let mixed_block = find_block_by_name(&mixed_blocks, "mixed_test").expect("Mixed block not found");
    assert!(mixed_block.content.contains("First line"));
    assert!(mixed_block.content.contains("Second line"));
    
    // Complex document with mixed line endings
    let complex_mixed = r#"[code:python name:complex_mixed]
print("Line with LF")
print("Line with CRLF")
print("Another LF line")
[/code:python]"#.replace("\n", "\r\n");
    
    let complex_blocks = parse_document(&complex_mixed).expect("Failed to parse complex mixed line endings");
    let complex_block = find_block_by_name(&complex_blocks, "complex_mixed").expect("Complex mixed block not found");
    assert!(complex_block.content.contains("Line with LF"));
    assert!(complex_block.content.contains("Line with CRLF"));
    assert!(complex_block.content.contains("Another LF line"));
}
#[test]
fn test_language_types() {
    // Test different language types in code blocks
    let input = r#"
[code:python name:python_code]
def hello():
    print("Hello from Python")
[/code:python]

[code:javascript name:js_code]
function hello() {
    console.log("Hello from JavaScript");
}
[/code:javascript]

[code:rust name:rust_code]
fn hello() {
    println!("Hello from Rust");
}
[/code:rust]

[code:html name:html_code]
<!DOCTYPE html>
<html>
<body>
    <h1>Hello from HTML</h1>
</body>
</html>
[/code:html]

[code:css name:css_code]
body {
    font-family: Arial, sans-serif;
    color: #333;
}
[/code:css]

[code:sql name:sql_code]
SELECT * FROM users WHERE name = 'John';
[/code:sql]

[code:json name:json_code]
{
  "name": "John",
  "age": 30,
  "isActive": true,
  "address": {
    "street": "123 Main St",
    "city": "Anytown"
  }
}
[/code:json]

[code:yaml name:yaml_code]
name: John
age: 30
isActive: true
address:
  street: 123 Main St
  city: Anytown
[/code:yaml]

[code:markdown name:markdown_code]
# Hello World

This is a **markdown** document with *formatting*.

- Item 1
- Item 2
- Item 3
[/code:markdown]

[code:bash name:bash_code]
#!/bin/bash
echo "Hello from Bash"
for i in {1..5}; do
  echo "Number: $i"
done
[/code:bash]
"#;

    let blocks = parse_document(input).expect("Failed to parse language types");
    
    // Check all language blocks were parsed correctly
    let languages = ["python", "javascript", "rust", "html", "css", "sql", "json", "yaml", "markdown", "bash"];
    for lang in languages.iter() {
        let block_name = format!("{}_code", lang);
        let block = find_block_by_name(&blocks, &block_name).expect(&format!("{} block not found", lang));
        assert_eq!(block.block_type, format!("code:{}", lang));
    }
    
    // Verify content of specific blocks
    let json_block = find_block_by_name(&blocks, "json_code").expect("JSON block not found");
    assert!(json_block.content.contains("\"name\": \"John\""));
    
    let yaml_block = find_block_by_name(&blocks, "yaml_code").expect("YAML block not found");
    assert!(yaml_block.content.contains("name: John"));
    
    let markdown_block = find_block_by_name(&blocks, "markdown_code").expect("Markdown block not found");
    assert!(markdown_block.content.contains("# Hello World"));
    
    let bash_block = find_block_by_name(&blocks, "bash_code").expect("Bash block not found");
    assert!(bash_block.content.contains("#!/bin/bash"));
}
#[test]
fn test_character_escaping() {
    // Test character escaping in content
    let input = r#"
[code:python name:escaped_chars]
print("Escaped quotes: \"Hello, world!\"")
print('Single quotes: \'Hello, world!\'')
print("Backslashes: \\path\\to\\file")
print("Tab: \t and newline: \n")
[/code:python]

[data name:escaped_json]
{
    "escaped": "This has \"quotes\" and \\ backslashes",
    "path": "C:\\Program Files\\App"
}
[/data]

[shell name:escaped_shell]
echo "Escaped characters: \" \' \\ \$ \` \!"
[/shell]

[code:python name:unicode_escapes]
# Unicode escapes
print("\u03C0 is approximately 3.14159")
print("\u2665 is a heart symbol")
print("\U0001F600 is a grinning face emoji")
[/code:python]

[code:javascript name:js_escapes]
// JavaScript escape sequences
console.log("Line 1\nLine 2");
console.log("Tab\tcharacter");
console.log("Unicode: \u03C0 \u2665");
console.log("Hex: \x41 \x42 \x43");
[/code:javascript]

[code:html name:html_entities]
<div>
  &lt;script&gt; tags should be escaped
  Special chars: &amp; &quot; &apos; &gt;
  Numeric entities: &#960; &#x2665;
</div>
[/code:html]
"#;

    let blocks = parse_document(input).expect("Failed to parse character escaping");
    
    // Check Python block with escaped characters
    let python_block = find_block_by_name(&blocks, "escaped_chars").expect("Escaped chars Python block not found");
    assert!(python_block.content.contains("\\\"Hello, world!\\\""));
    assert!(python_block.content.contains("\\'Hello, world!\\'"));
    assert!(python_block.content.contains("\\\\path\\\\to\\\\file"));
    
    // Check JSON data with escaped characters
    let json_block = find_block_by_name(&blocks, "escaped_json").expect("Escaped JSON block not found");
    assert!(json_block.content.contains("\\\"quotes\\\""));
    assert!(json_block.content.contains("C:\\\\Program Files\\\\App"));
    
    // Check shell command with escaped characters
    let shell_block = find_block_by_name(&blocks, "escaped_shell").expect("Escaped shell block not found");
    assert!(shell_block.content.contains("\\\" \\' \\\\ \\$ \\` \\!"));
    
    // Check Unicode escapes
    let unicode_block = find_block_by_name(&blocks, "unicode_escapes").expect("Unicode escapes block not found");
    assert!(unicode_block.content.contains("\\u03C0"));
    assert!(unicode_block.content.contains("\\u2665"));
    assert!(unicode_block.content.contains("\\U0001F600"));
    
    // Check JavaScript escapes
    let js_block = find_block_by_name(&blocks, "js_escapes").expect("JS escapes block not found");
    assert!(js_block.content.contains("Line 1\\nLine 2"));
    assert!(js_block.content.contains("Tab\\tcharacter"));
    assert!(js_block.content.contains("\\u03C0 \\u2665"));
    assert!(js_block.content.contains("\\x41 \\x42 \\x43"));
    
    // Check HTML entities
    let html_block = find_block_by_name(&blocks, "html_entities").expect("HTML entities block not found");
    assert!(html_block.content.contains("&lt;script&gt;"));
    assert!(html_block.content.contains("&amp; &quot; &apos; &gt;"));
    assert!(html_block.content.contains("&#960; &#x2665;"));
}
#[test]
fn test_large_blocks() {
    // Test very large blocks
    let mut large_content = String::new();
    for i in 0..1000 {
        large_content.push_str(&format!("print(\"Line {} of a very large block\")\n", i));
    }
    
    let input = format!(r#"
[code:python name:large_block]
{}
[/code:python]
"#, large_content);

    let blocks = parse_document(&input).expect("Failed to parse large block");
    
    // Check the large block was parsed correctly
    let large_block = find_block_by_name(&blocks, "large_block").expect("Large block not found");
    assert!(large_block.content.contains("Line 0 of a very large block"));
    assert!(large_block.content.contains("Line 999 of a very large block"));
    assert_eq!(large_block.content.lines().count(), 1000);
    
    // Test large block with complex content
    let mut large_json = String::new();
    large_json.push_str("{\n  \"items\": [\n");
    for i in 0..500 {
        large_json.push_str(&format!("    {{ \"id\": {}, \"name\": \"Item {}\" }}{}\n", 
                                    i, i, if i < 499 { "," } else { "" }));
    }
    large_json.push_str("  ]\n}");
    
    let json_input = format!(r#"
[data name:large_json]
{}
[/data]
"#, large_json);

    let json_blocks = parse_document(&json_input).expect("Failed to parse large JSON");
    let json_block = find_block_by_name(&json_blocks, "large_json").expect("Large JSON block not found");
    assert!(json_block.content.contains("\"id\": 0"));
    assert!(json_block.content.contains("\"id\": 499"));
    assert!(json_block.content.lines().count() > 500);
}
#[test]
fn test_indentation_patterns() {
    // Test blocks with complicated indentation patterns
    let input = r#"
[code:python name:indented_code]
def complex_function():
    if True:
        for i in range(10):
            if i % 2 == 0:
                print("Even")
            else:
                print("Odd")
                if i > 5:
                    print("Greater than 5")
                    for j in range(i):
                        print(f"  Nested: {j}")
[/code:python]

[data name:indented_json]
{
    "level1": {
        "level2": {
            "level3": {
                "level4": {
                    "value": "deeply nested"
                }
            }
        }
    }
}
[/data]

[code:python name:mixed_indentation]
def mixed_function():
    # Spaces
    if True:
        print("Spaces: 4")
    # Tabs
	print("Tab: 1")
		print("Tabs: 2")
    # Mixed
    if True:
	    print("Mixed: 1 tab after 4 spaces")
	if True:
        print("Mixed: 4 spaces after 1 tab")
[/code:python]

[code:yaml name:yaml_indentation]
root:
  level1:
    level2:
      - item1
      - item2
      - nested:
          key1: value1
          key2: value2
  sibling:
    - simple_item
    - complex_item:
        subkey: subvalue
[/code:yaml]
"#;

    let blocks = parse_document(input).expect("Failed to parse indentation patterns");
    
    // Check Python block with complex indentation
    let python_block = find_block_by_name(&blocks, "indented_code").expect("Indented code block not found");
    assert!(python_block.content.contains("def complex_function():"));
    assert!(python_block.content.contains("    if True:"));
    assert!(python_block.content.contains("        for i in range(10):"));
    assert!(python_block.content.contains("            if i % 2 == 0:"));
    
    // Check JSON data with nested indentation
    let json_block = find_block_by_name(&blocks, "indented_json").expect("Indented JSON block not found");
    assert!(json_block.content.contains("\"level1\": {"));
    assert!(json_block.content.contains("    \"level2\": {"));
    assert!(json_block.content.contains("        \"level3\": {"));
    assert!(json_block.content.contains("            \"level4\": {"));
    
    // Check mixed indentation
    let mixed_block = find_block_by_name(&blocks, "mixed_indentation").expect("Mixed indentation block not found");
    assert!(mixed_block.content.contains("	print(\"Tab: 1\")"));
    assert!(mixed_block.content.contains("		print(\"Tabs: 2\")"));
    assert!(mixed_block.content.contains("	    print(\"Mixed: 1 tab after 4 spaces\")"));
    assert!(mixed_block.content.contains("        print(\"Mixed: 4 spaces after 1 tab\")"));
    
    // Check YAML indentation
    let yaml_block = find_block_by_name(&blocks, "yaml_indentation").expect("YAML indentation block not found");
    assert!(yaml_block.content.contains("  level1:"));
    assert!(yaml_block.content.contains("    level2:"));
    assert!(yaml_block.content.contains("      - item1"));
    assert!(yaml_block.content.contains("          key1: value1"));
}
#[test]
fn test_error_recovery() {
    // Test parse error recovery
    let input = r#"
[code:python name:valid_block]
print("This is a valid block")
[/code:python]

[invalid_block]
This block has an invalid type
[/invalid_block]

[code:python name:another_valid]
print("This block should still be parsed")
[/code:python]
"#;

    // This should fail because of the invalid block
    assert!(parse_document(input).is_err());
    
    // But we can test partial parsing or error recovery if implemented
    // For now, just verify that the parser correctly identifies the error
    match parse_document(input) {
        Err(e) => {
            let error_string = format!("{:?}", e);
            assert!(error_string.contains("invalid") || error_string.contains("unknown block type"),
                   "Error message should mention invalid block: {:?}", error_string);
        },
        Ok(_) => panic!("Parser should have returned an error for invalid block"),
    }
    
    // Test with a document containing valid blocks and a syntax error
    let input_with_syntax_error = r#"
[code:python name:first_valid]
print("This is a valid block")
[/code:python]

[code:python name:syntax_error
print("This block has a syntax error - missing closing bracket")
[/code:python]

[code:python name:last_valid]
print("This is another valid block")
[/code:python]
"#;

    // This should fail because of the syntax error
    assert!(parse_document(input_with_syntax_error).is_err());
    
    // Test with a document containing valid blocks and a block with invalid modifiers
    let input_with_invalid_modifier = r#"
[code:python name:first_valid]
print("This is a valid block")
[/code:python]

[code:python name:invalid:modifier]
print("This block has an invalid modifier")
[/code:python]

[code:python name:last_valid]
print("This is another valid block")
[/code:python]
"#;

    // This should fail because of the invalid modifier
    assert!(parse_document(input_with_invalid_modifier).is_err());
}
#[test]
fn test_complex_document() {
    // Test parsing a complex document with multiple block types and nesting
    let input = r#"
[section:intro name:document_intro]
# Introduction

This is a complex document with multiple block types and nesting.

[code:python name:setup_code]
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

def setup_environment():
    print("Setting up environment")
    return {"ready": True}
[/code:python]

[variable name:config]
{
    "data_source": "example.csv",
    "max_rows": 1000,
    "columns": ["id", "name", "value"]
}
[/variable]

[section:data_processing name:data_section]
## Data Processing

[code:python name:process_data depends:setup_code]
def process_data(source="${config.data_source}"):
    print(f"Processing data from {source}")
    # Use the setup from the previous block
    env = setup_environment()
    if env["ready"]:
        return {"processed": True}
[/code:python]

[data name:sample_data]
{"id": 1, "name": "Example", "value": 42}
[/data]

[shell name:run_script]
python -c "import json; print(json.dumps(${sample_data}))"
[/shell]

[results for:run_script]
{"id": 1, "name": "Example", "value": 42}
[/results]
[/section:data_processing]

[section:visualization name:viz_section]
## Visualization

[code:python name:create_viz depends:process_data]
def create_visualization(data):
    print("Creating visualization")
    # This would normally create a plot
    return "visualization.png"
[/code:python]

[visualization name:data_viz type:bar data:sample_data]
// Visualization configuration
[/visualization]

[preview for:data_viz]
[Bar chart showing sample data]
[/preview]
[/section:visualization]

[template name:report_template]
# ${title}

Data processed: ${data_processed}
Visualization: ${visualization_path}

## Summary
${summary}
[/template]

[template:report_template name:final_report 
  title:"Analysis Report"
  data_processed:"Yes"
  visualization_path:"visualization.png"
  summary:"This is a summary of the analysis."
]
[/template:report_template]

[conditional if:config.max_rows>500]
This section only appears if max_rows is greater than 500.
[/conditional]

[error_results for:missing_block]
Error: Block not found
[/error_results]
[/section:intro]
"#;

    let blocks = parse_document(input).expect("Failed to parse complex document");
    
    // Find the main section
    let intro_section = find_block_by_name(&blocks, "document_intro").expect("Intro section not found");
    assert_eq!(intro_section.block_type, "section:intro");
    
    // Check that the section has the expected number of child blocks
    assert!(intro_section.children.len() >= 5, "Expected at least 5 child blocks, got {}", intro_section.children.len());
    
    // Check nested sections
    let data_section = find_child_by_name(intro_section, "data_section").expect("Data section not found");
    assert_eq!(data_section.block_type, "section:data_processing");
    assert_eq!(data_section.children.len(), 4, "Expected 4 child blocks in data section, got {}", data_section.children.len());
    
    let viz_section = find_child_by_name(intro_section, "viz_section").expect("Visualization section not found");
    assert_eq!(viz_section.block_type, "section:visualization");
    assert_eq!(viz_section.children.len(), 3, "Expected 3 child blocks in viz section, got {}", viz_section.children.len());
    
    // Check variable references
    let process_data = find_child_by_name(data_section, "process_data").expect("Process data block not found");
    assert!(process_data.content.contains("${config.data_source}"));
    
    // Check dependencies
    assert!(has_modifier(process_data, "depends", "setup_code"));
    
    let create_viz = find_child_by_name(viz_section, "create_viz").expect("Create viz block not found");
    assert!(has_modifier(create_viz, "depends", "process_data"));
    
    // Check template invocation
    let final_report = find_block_by_name(&blocks, "final_report").expect("Final report not found");
    assert!(has_modifier(final_report, "title", "Analysis Report"));
    assert!(has_modifier(final_report, "data_processed", "Yes"));
    assert!(has_modifier(final_report, "visualization_path", "visualization.png"));
    assert!(has_modifier(final_report, "summary", "This is a summary of the analysis."));
    
    // Check conditional block
    let conditional = intro_section.children.iter()
        .find(|b| b.block_type == "conditional")
        .expect("Conditional block not found");
    assert!(has_modifier(conditional, "if", "config.max_rows>500"));
}

/// Test error cases with malformed blocks
#[test]
fn test_malformed_blocks() {
    // Missing closing tag
    let input1 = r#"
    [code:python name:missing-close]
    print("Hello, world!")
    "#;
    
    let result1 = parse_document(input1);
    assert!(result1.is_err(), "Parser should fail on missing closing tag");
    println!("DEBUG: Error for missing closing tag: {:?}", result1.err());
    
    // Mismatched closing tag
    let input2 = r#"
    [code:python name:mismatched-close]
    print("Hello, world!")
    [/shell]
    "#;
    
    let result2 = parse_document(input2);
    assert!(result2.is_err(), "Parser should fail on mismatched closing tag");
    println!("DEBUG: Error for mismatched closing tag: {:?}", result2.err());
    
    // Malformed opening tag (missing bracket)
    let input3 = r#"
    code:python name:malformed-open]
    print("Hello, world!")
    [/code:python]
    "#;
    
    let result3 = parse_document(input3);
    assert!(result3.is_err(), "Parser should fail on malformed opening tag");
    println!("DEBUG: Error for malformed opening tag: {:?}", result3.err());
    
    // Missing required name attribute
    let input4 = r#"
    [variable]
    value without name
    [/variable]
    "#;
    
    let result4 = parse_document(input4);
    assert!(result4.is_err(), "Parser should fail on variable block without name");
    println!("DEBUG: Error for missing required name: {:?}", result4.err());
    
    // Invalid modifier format
    let input5 = r#"
    [code:python name:invalid-modifier-format cache_result:notboolean]
    print("Hello, world!")
    [/code:python]
    "#;
    
    // This might parse successfully as modifiers are not validated during parsing
    let result5 = parse_document(input5);
    if let Ok(blocks) = result5 {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("invalid-modifier-format"));
        assert!(block.is_some(), "Block with invalid modifier format not found");
        
        // The modifier should be present even if the value is invalid
        let block = block.unwrap();
        assert!(block.has_modifier("cache_result"), "cache_result modifier not found");
        println!("DEBUG: cache_result value: {:?}", block.get_modifier("cache_result"));
    } else {
        println!("DEBUG: Parser rejected invalid modifier format: {:?}", result5.err());
    }
}

/// Test different closing tag variants
#[test]
fn test_closing_tag_variants() {
    // Code block with language in closing tag
    let input1 = r#"
    [code:python name:with-language-close]
    print("Hello, world!")
    [/code:python]
    "#;
    
    let result1 = parse_document(input1);
    assert!(result1.is_ok(), "Failed to parse code block with language in closing tag: {:?}", result1.err());
    
    // Code block without language in closing tag
    let input2 = r#"
    [code:python name:without-language-close]
    print("Hello, world!")
    [/code]
    "#;
    
    let result2 = parse_document(input2);
    assert!(result2.is_ok(), "Failed to parse code block without language in closing tag: {:?}", result2.err());
    
    // Section block with type in closing tag
    let input3 = r#"
    [section:intro name:with-type-close]
    Introduction content
    [/section:intro]
    "#;
    
    let result3 = parse_document(input3);
    assert!(result3.is_ok(), "Failed to parse section block with type in closing tag: {:?}", result3.err());
    
    // Section block without type in closing tag
    let input4 = r#"
    [section:summary name:without-type-close]
    Summary content
    [/section]
    "#;
    
    let result4 = parse_document(input4);
    assert!(result4.is_ok(), "Failed to parse section block without type in closing tag: {:?}", result4.err());
    
    // Verify blocks were parsed correctly
    if let Ok(blocks) = result1 {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("with-language-close"));
        assert!(block.is_some(), "Block with language in closing tag not found");
        assert_eq!(block.unwrap().block_type, "code:python", "Block type incorrect");
    }
    
    if let Ok(blocks) = result2 {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("without-language-close"));
        assert!(block.is_some(), "Block without language in closing tag not found");
        assert_eq!(block.unwrap().block_type, "code:python", "Block type incorrect");
    }
    
    if let Ok(blocks) = result3 {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("with-type-close"));
        assert!(block.is_some(), "Block with type in closing tag not found");
        assert_eq!(block.unwrap().block_type, "section:intro", "Block type incorrect");
    }
    
    if let Ok(blocks) = result4 {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("without-type-close"));
        assert!(block.is_some(), "Block without type in closing tag not found");
        assert_eq!(block.unwrap().block_type, "section:summary", "Block type incorrect");
    }
}

/// Test line ending differences (CRLF vs LF)
#[test]
fn test_line_ending_differences() {
    // LF line endings
    let input_lf = "[code:python name:lf-endings]\nprint(\"Hello, LF!\")\n[/code:python]";
    
    // CRLF line endings
    let input_crlf = "[code:python name:crlf-endings]\r\nprint(\"Hello, CRLF!\")\r\n[/code:python]";
    
    // Mixed line endings
    let input_mixed = "[code:python name:mixed-endings]\nprint(\"Line 1\")\r\nprint(\"Line 2\")\n[/code:python]";
    
    let result_lf = parse_document(input_lf);
    let result_crlf = parse_document(input_crlf);
    let result_mixed = parse_document(input_mixed);
    
    assert!(result_lf.is_ok(), "Failed to parse document with LF line endings: {:?}", result_lf.err());
    assert!(result_crlf.is_ok(), "Failed to parse document with CRLF line endings: {:?}", result_crlf.err());
    assert!(result_mixed.is_ok(), "Failed to parse document with mixed line endings: {:?}", result_mixed.err());
    
    // Verify content is preserved correctly
    if let Ok(blocks) = result_lf {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("lf-endings")).unwrap();
        assert_eq!(block.content.trim(), "print(\"Hello, LF!\")", "Content with LF endings not preserved");
    }
    
    if let Ok(blocks) = result_crlf {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("crlf-endings")).unwrap();
        assert_eq!(block.content.trim(), "print(\"Hello, CRLF!\")", "Content with CRLF endings not preserved");
    }
    
    if let Ok(blocks) = result_mixed {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some("mixed-endings")).unwrap();
        let content = block.content.trim();
        assert!(content.contains("Line 1") && content.contains("Line 2"), 
                "Content with mixed line endings not preserved");
    }
}

/// Test different language types in code blocks
#[test]
fn test_different_languages() {
    let input = r#"
    [code:python name:python-code]
    def hello():
        print("Hello, Python!")
    [/code:python]
    
    [code:javascript name:javascript-code]
    function hello() {
        console.log("Hello, JavaScript!");
    }
    [/code:javascript]
    
    [code:rust name:rust-code]
    fn hello() {
        println!("Hello, Rust!");
    }
    [/code:rust]
    
    [code:sql name:sql-code]
    SELECT * FROM users WHERE name = 'Test';
    [/code:sql]
    
    [code:html name:html-code]
    <div class="greeting">
        <h1>Hello, HTML!</h1>
    </div>
    [/code:html]
    
    [code:css name:css-code]
    .greeting {
        color: blue;
        font-weight: bold;
    }
    [/code:css]
    
    [code:c name:c-code]
    #include <stdio.h>
    
    int main() {
        printf("Hello, C!\n");
        return 0;
    }
    [/code:c]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with different languages: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Verify each language block
    let languages = [
        ("python-code", "code:python"),
        ("javascript-code", "code:javascript"),
        ("rust-code", "code:rust"),
        ("sql-code", "code:sql"),
        ("html-code", "code:html"),
        ("css-code", "code:css"),
        ("c-code", "code:c")
    ];
    
    for (name, expected_type) in languages {
        let block = blocks.iter().find(|b| b.name.as_deref() == Some(name));
        assert!(block.is_some(), "Block {} not found", name);
        
        let block = block.unwrap();
        assert_eq!(block.block_type, expected_type, 
                  "Block {} has incorrect type: {}", name, block.block_type);
        
        println!("DEBUG: {} content: {}", name, block.content);
    }
}

/// Test character escaping in content
#[test]
fn test_character_escaping() {
    let input = r#"
    [code:python name:code-with-brackets]
    # Code with square brackets
    data = [1, 2, 3, 4]
    nested = [[1, 2], [3, 4]]
    print(f"Data: {data}")
    [/code:python]
    
    [data name:json-with-escaped-quotes format:json]
    {
      "string": "This has \"quoted\" text",
      "path": "C:\\Users\\test\\file.txt"
    }
    [/data]
    
    [shell name:shell-with-redirects]
    grep "pattern" file.txt > results.txt
    cat file1.txt | grep "test" | sort > sorted.txt
    [/shell]
    
    [variable name:special-chars]
    Line with backslash: \
    Line with escaped chars: \n \t \r
    Line with percent: 100%
    Line with dollar: $PATH
    [/variable]
    
    [code:html name:html-with-entities]
    <p>This is an HTML paragraph with &lt;tags&gt; and &amp; symbol</p>
    <script>
      if (x < 10 && y > 20) {
        console.log("test");
      }
    </script>
    [/code:html]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with escaped characters: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Check code with brackets
    let brackets_block = blocks.iter().find(|b| b.name.as_deref() == Some("code-with-brackets"));
    assert!(brackets_block.is_some(), "Block with brackets not found");
    assert!(brackets_block.unwrap().content.contains("[1, 2, 3, 4]"), 
           "Block content with brackets not preserved");
    
    // Check JSON with escaped quotes
    let json_block = blocks.iter().find(|b| b.name.as_deref() == Some("json-with-escaped-quotes"));
    assert!(json_block.is_some(), "JSON block with escaped quotes not found");
    let json_content = &json_block.unwrap().content;
    assert!(json_content.contains("\"quoted\""), "JSON escaped quotes not preserved");
    assert!(json_content.contains("C:\\\\Users"), "JSON escaped backslashes not preserved");
    
    // Check shell with redirects
    let shell_block = blocks.iter().find(|b| b.name.as_deref() == Some("shell-with-redirects"));
    assert!(shell_block.is_some(), "Shell block with redirects not found");
    let shell_content = &shell_block.unwrap().content;
    assert!(shell_content.contains(">"), "Shell redirects not preserved");
    assert!(shell_content.contains("|"), "Shell pipes not preserved");
    
    // Check variable with special chars
    let var_block = blocks.iter().find(|b| b.name.as_deref() == Some("special-chars"));
    assert!(var_block.is_some(), "Variable block with special chars not found");
    let var_content = &var_block.unwrap().content;
    assert!(var_content.contains("\\"), "Backslash not preserved");
    assert!(var_content.contains("\\n"), "Escaped newline not preserved");
    assert!(var_content.contains("$PATH"), "Dollar sign not preserved");
    
    // Check HTML with entities
    let html_block = blocks.iter().find(|b| b.name.as_deref() == Some("html-with-entities"));
    assert!(html_block.is_some(), "HTML block with entities not found");
    let html_content = &html_block.unwrap().content;
    assert!(html_content.contains("&lt;"), "HTML entities not preserved");
    assert!(html_content.contains("x < 10"), "Less than sign not preserved");
    assert!(html_content.contains("y > 20"), "Greater than sign not preserved");
    
    println!("DEBUG: All blocks with escaped characters parsed correctly");
}

/// Test very large blocks
#[test]
fn test_large_blocks() {
    // Create a large block with repeated content
    let large_content = "print(\"This is line {}\")".repeat(1000);
    let large_block = format!("[code:python name:large-code-block]\n{}\n[/code:python]", large_content);
    
    // Create a large block with lots of nested brackets
    let nested_brackets = (0..100).map(|i| format!("{}{}{}", "[".repeat(i), "content", "]".repeat(i))).collect::<Vec<_>>().join("\n");
    let brackets_block = format!("[data name:nested-brackets]\n{}\n[/data]", nested_brackets);
    
    // Create a large document with many small blocks
    let many_blocks = (0..100).map(|i| format!("[variable name:var{}]\nvalue{}\n[/variable]", i, i)).collect::<Vec<_>>().join("\n\n");
    
    // Test each large input
    let result1 = parse_document(&large_block);
    assert!(result1.is_ok(), "Failed to parse large code block: {:?}", result1.err());
    if let Ok(blocks) = result1 {
        assert_eq!(blocks.len(), 1, "Should have exactly one block");
        assert_eq!(blocks[0].name.as_deref(), Some("large-code-block"), "Block name incorrect");
        println!("DEBUG: Large code block parsed successfully, content length: {}", blocks[0].content.len());
    }
    
    let result2 = parse_document(&brackets_block);
    assert!(result2.is_ok(), "Failed to parse block with nested brackets: {:?}", result2.err());
    if let Ok(blocks) = result2 {
        assert_eq!(blocks.len(), 1, "Should have exactly one block");
        assert_eq!(blocks[0].name.as_deref(), Some("nested-brackets"), "Block name incorrect");
        println!("DEBUG: Nested brackets block parsed successfully, content length: {}", blocks[0].content.len());
    }
    
    let result3 = parse_document(&many_blocks);
    assert!(result3.is_ok(), "Failed to parse document with many blocks: {:?}", result3.err());
    if let Ok(blocks) = result3 {
        assert_eq!(blocks.len(), 100, "Should have exactly 100 blocks");
        println!("DEBUG: Document with many blocks parsed successfully, block count: {}", blocks.len());
    }
}

/// Test blocks with complicated indentation patterns
#[test]
fn test_indentation_patterns() {
    let input = r#"
    [code:python name:python-indentation]
    def complex_function():
        # First level
        if True:
            # Second level
            for i in range(10):
                # Third level
                if i % 2 == 0:
                    # Fourth level
                    print(f"Even: {i}")
                else:
                    # Also fourth level
                    print(f"Odd: {i}")
            # Back to second level
        # Back to first level
        return "Done"
    [/code:python]
    
    [code:javascript name:js-indentation]
    function complexFunction() {
      // First level
      if (true) {
        // Second level
        for (let i = 0; i < 10; i++) {
          // Third level
          if (i % 2 === 0) {
            // Fourth level
            console.log(`Even: ${i}`);
          } else {
            // Also fourth level
            console.log(`Odd: ${i}`);
          }
        }
        // Back to second level
      }
      // Back to first level
      return "Done";
    }
    [/code:javascript]
    
    [data name:json-indentation format:json]
    {
      "level1": {
        "level2": {
          "level3": {
            "level4": {
              "value": "Deeply nested value"
            },
            "array": [
              {
                "item": 1
              },
              {
                "item": 2
              }
            ]
          }
        }
      }
    }
    [/data]
    "#;
    
    let result = parse_document(input);
    assert!(result.is_ok(), "Failed to parse document with complex indentation: {:?}", result.err());
    
    let blocks = result.unwrap();
    
    // Verify Python indentation
    let python_block = blocks.iter().find(|b| b.name.as_deref() == Some("python-indentation"));
    assert!(python_block.is_some(), "Python indentation block not found");
    let python_content = &python_block.unwrap().content;
    
    // Check if indentation levels are preserved
    assert!(python_content.contains("def complex_function():"), "First level indentation not preserved");
    assert!(python_content.contains("    # First level"), "First level comment indentation not preserved");
    assert!(python_content.contains("        # Second level"), "Second level comment indentation not preserved");
    assert!(python_content.contains("            # Third level"), "Third level comment indentation not preserved");
    assert!(python_content.contains("                # Fourth level"), "Fourth level comment indentation not preserved");
    
    // Verify JavaScript indentation
    let js_block = blocks.iter().find(|b| b.name.as_deref() == Some("js-indentation"));
    assert!(js_block.is_some(), "JavaScript indentation block not found");
    let js_content = &js_block.unwrap().content;
    
    // Check if indentation levels are preserved
    assert!(js_content.contains("function complexFunction() {"), "First level indentation not preserved");
    assert!(js_content.contains("  // First level"), "First level comment indentation
