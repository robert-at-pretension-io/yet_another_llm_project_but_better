//! Parser robustness tests
//! 
//! This file contains comprehensive tests for the Meta Language parser,
//! focusing on edge cases, complex structures, and error handling.

use yet_another_llm_project_but_better::parser::{parse_document, Block, ParserError};

/// Test basic block parsing for each block type
#[test]
fn test_basic_block_types() {
    let input = r#"
    [question name:test-question]
    What is the meaning of life?
    [/question]
    
    [response name:test-response]
    The meaning of life is 42.
    [/response]
    
    [code:python name:test-code]
    print("Hello, world!")
    [/code:python]
    
    [shell name:test-shell]
    echo "Hello from shell"
    [/shell]
    
    [api name:test-api]
    https://api.example.com/data
    [/api]
    
    [data name:test-data]
    {"key": "value"}
    [/data]
    
    [variable name:test-variable]
    test-value
    [/variable]
    
    [secret name:test-secret]
    API_KEY
    [/secret]
    
    [template name:test-template]
    Template content
    [/template]
    
    [error name:test-error]
    Error message
    [/error]
    
    [visualization name:test-visualization]
    Visualization content
    [/visualization]
    
    [preview name:test-preview]
    Preview content
    [/preview]
    
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
