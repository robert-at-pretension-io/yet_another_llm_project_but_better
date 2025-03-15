use std::collections::HashMap;
use std::path::Path;
use std::fs;

// Import the parser module and related types
use metalanguage::parser::{parse_document, Block};
use metalanguage::parser::ParserError;

/// Helper function to parse a string and return the blocks
fn parse_test_input(input: &str) -> Result<Vec<Block>, ParserError> {
    parse_document(input)
}

/// Helper function to find a block by name in a list of blocks
fn find_block_by_name<'a>(blocks: &'a [Block], name: &str) -> Option<&'a Block> {
    blocks.iter().find(|b| b.name.as_ref().map_or(false, |n| n == name))
}

/// Helper function to find a block by type in a list of blocks
fn find_blocks_by_type<'a>(blocks: &'a [Block], block_type: &str) -> Vec<&'a Block> {
    blocks.iter().filter(|b| b.block_type == block_type).collect()
}

/// Helper function to check if a block has a specific modifier
fn has_modifier(block: &Block, key: &str, value: &str) -> bool {
    block.modifiers.iter().any(|(k, v)| k == key && v == value)
}

#[test]
fn test_basic_block_types() {
    // Test all basic block types
    let input = r#"
[code:python name:basic_python]
print("Hello, world!")
[/code:python]

[data name:basic_data]
{"key": "value"}
[/data]

[shell name:basic_shell]
echo "Hello from shell"
[/shell]

[visualization name:basic_viz type:bar]
[/visualization]

[variable name:basic_var]
some variable content
[/variable]

[secret name:basic_secret]
password123
[/secret]

[filename name:basic_filename]
/path/to/file.txt
[/filename]

[memory name:basic_memory]
Memory content
[/memory]

[api name:basic_api url:https://api.example.com]
[/api]

[question name:basic_question]
What is your name?
[/question]

[response name:basic_response for:basic_question]
My name is Bot.
[/response]

[results name:basic_results for:basic_python]
Hello, world!
[/results]

[error_results name:basic_error for:basic_shell]
Command failed: Permission denied
[/error_results]

[error name:basic_error]
This is an error message
[/error]

[preview name:basic_preview for:basic_data]
{"key": "value"}
[/preview]

[conditional name:basic_conditional if:basic_var]
Conditional content
[/conditional]
"#;

    let blocks = parse_test_input(input).expect("Failed to parse basic blocks");
    
    // Verify we have the expected number of blocks
    assert_eq!(blocks.len(), 17, "Expected 17 blocks, got {}", blocks.len());
    
    // Check each block type exists
    let block_types = [
        "code:python", "data", "shell", "visualization", "variable", 
        "secret", "filename", "memory", "api", "question", 
        "response", "results", "error_results", "error", "preview", "conditional"
    ];
    
    for block_type in block_types.iter() {
        assert!(
            blocks.iter().any(|b| b.block_type == *block_type),
            "Missing block type: {}", block_type
        );
    }
    
    // Verify specific block content
    let python_block = find_block_by_name(&blocks, "basic_python").expect("Python block not found");
    assert_eq!(python_block.content.trim(), "print(\"Hello, world!\")");
    
    let data_block = find_block_by_name(&blocks, "basic_data").expect("Data block not found");
    assert_eq!(data_block.content.trim(), "{\"key\": \"value\"}");
}

#[test]
fn test_nested_blocks() {
    // Test nested blocks inside sections
    let input = r#"
[section:intro name:nested_section]
This is an introduction section.

[code:python name:nested_code]
def hello():
    print("Hello from nested code")
[/code:python]

[data name:nested_data]
{"nested": true}
[/data]

More section content.
[/section:intro]
"#;

    let blocks = parse_test_input(input).expect("Failed to parse nested blocks");
    
    // Find the section block
    let section_block = find_block_by_name(&blocks, "nested_section").expect("Section block not found");
    assert_eq!(section_block.block_type, "section:intro");
    
    // Check that the section has child blocks
    assert_eq!(section_block.children.len(), 2, "Expected 2 child blocks, got {}", section_block.children.len());
    
    // Verify the nested blocks
    let nested_code = section_block.children.iter()
        .find(|b| b.name.as_ref().map_or(false, |n| n == "nested_code"))
        .expect("Nested code block not found");
    assert_eq!(nested_code.block_type, "code:python");
    
    let nested_data = section_block.children.iter()
        .find(|b| b.name.as_ref().map_or(false, |n| n == "nested_data"))
        .expect("Nested data block not found");
    assert_eq!(nested_data.block_type, "data");
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse complex modifiers");
    
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse variable references");
    
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse template invocations");
    
    // Check template definitions
    let template_blocks = find_blocks_by_type(&blocks, "template");
    assert_eq!(template_blocks.len(), 2, "Expected 2 template blocks, got {}", template_blocks.len());
    
    // Check template invocations
    let invocation_blocks = blocks.iter()
        .filter(|b| b.block_type == "template_invocation" || b.block_type.starts_with("template_invocation:"))
        .collect::<Vec<_>>();
    assert_eq!(invocation_blocks.len(), 3, "Expected 3 template invocation blocks, got {}", invocation_blocks.len());
    
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse special characters");
    
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse whitespace patterns");
    
    // Check blocks with whitespace in tags
    assert!(find_block_by_name(&blocks, "whitespace_test").is_some());
    assert!(find_block_by_name(&blocks, "whitespace_data").is_some());
    
    // Check indented code block
    let code_block = find_block_by_name(&blocks, "whitespace_test").expect("Whitespace code block not found");
    assert!(code_block.content.contains("    print(\"Indented code\")"));
    
    // Check block with empty lines
    let empty_lines = find_block_by_name(&blocks, "empty_lines").expect("Empty lines block not found");
    assert_eq!(empty_lines.content.trim(), "Variable with empty lines");
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse multiple modifiers");
    
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
}

#[test]
fn test_malformed_blocks() {
    // Test error cases with malformed blocks
    
    // Missing closing tag
    let input1 = r#"
[code:python name:missing_close]
print("This block has no closing tag")
"#;
    assert!(parse_test_input(input1).is_err());
    
    // Mismatched closing tag
    let input2 = r#"
[code:python name:mismatched_close]
print("This block has mismatched closing tag")
[/code:javascript]
"#;
    assert!(parse_test_input(input2).is_err());
    
    // Invalid modifier format
    let input3 = r#"
[code:python name:invalid:modifier]
print("This block has an invalid modifier")
[/code:python]
"#;
    assert!(parse_test_input(input3).is_err());
    
    // Duplicate block names
    let input4 = r#"
[code:python name:duplicate]
print("First block")
[/code:python]

[data name:duplicate]
{"duplicate": true}
[/data]
"#;
    assert!(parse_test_input(input4).is_err());
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse closing tags");
    
    // Check all blocks were parsed correctly
    assert!(find_block_by_name(&blocks, "with_lang").is_some());
    assert!(find_block_by_name(&blocks, "without_lang").is_some());
    assert!(find_block_by_name(&blocks, "js_block").is_some());
    assert!(find_block_by_name(&blocks, "js_without_lang").is_some());
    
    // Verify content of blocks
    let with_lang = find_block_by_name(&blocks, "with_lang").expect("With lang block not found");
    assert_eq!(with_lang.content.trim(), "print(\"Block with language in closing tag\")");
    
    let without_lang = find_block_by_name(&blocks, "without_lang").expect("Without lang block not found");
    assert_eq!(without_lang.content.trim(), "print(\"Block without language in closing tag\")");
}

#[test]
fn test_line_endings() {
    // Test CRLF vs LF line ending differences
    
    // LF line endings
    let lf_input = "[code:python name:lf_test]\nprint(\"LF line endings\")\n[/code:python]";
    let lf_blocks = parse_test_input(lf_input).expect("Failed to parse LF line endings");
    let lf_block = find_block_by_name(&lf_blocks, "lf_test").expect("LF block not found");
    assert_eq!(lf_block.content.trim(), "print(\"LF line endings\")");
    
    // CRLF line endings
    let crlf_input = "[code:python name:crlf_test]\r\nprint(\"CRLF line endings\")\r\n[/code:python]";
    let crlf_blocks = parse_test_input(crlf_input).expect("Failed to parse CRLF line endings");
    let crlf_block = find_block_by_name(&crlf_blocks, "crlf_test").expect("CRLF block not found");
    assert_eq!(crlf_block.content.trim(), "print(\"CRLF line endings\")");
    
    // Mixed line endings
    let mixed_input = "[code:python name:mixed_test]\nprint(\"First line\")\r\nprint(\"Second line\")\n[/code:python]";
    let mixed_blocks = parse_test_input(mixed_input).expect("Failed to parse mixed line endings");
    let mixed_block = find_block_by_name(&mixed_blocks, "mixed_test").expect("Mixed block not found");
    assert!(mixed_block.content.contains("First line"));
    assert!(mixed_block.content.contains("Second line"));
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse language types");
    
    // Check all language blocks were parsed correctly
    let languages = ["python", "javascript", "rust", "html", "css", "sql"];
    for lang in languages.iter() {
        let block_name = format!("{}_code", lang);
        let block = find_block_by_name(&blocks, &block_name).expect(&format!("{} block not found", lang));
        assert_eq!(block.block_type, format!("code:{}", lang));
    }
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse character escaping");
    
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

    let blocks = parse_test_input(&input).expect("Failed to parse large block");
    
    // Check the large block was parsed correctly
    let large_block = find_block_by_name(&blocks, "large_block").expect("Large block not found");
    assert!(large_block.content.contains("Line 0 of a very large block"));
    assert!(large_block.content.contains("Line 999 of a very large block"));
    assert_eq!(large_block.content.lines().count(), 1000);
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
"#;

    let blocks = parse_test_input(input).expect("Failed to parse indentation patterns");
    
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
    assert!(parse_test_input(input).is_err());
    
    // But we can test partial parsing or error recovery if implemented
    // For now, just verify that the parser correctly identifies the error
    match parse_test_input(input) {
        Err(e) => {
            let error_string = format!("{:?}", e);
            assert!(error_string.contains("invalid") || error_string.contains("unknown block type"),
                   "Error message should mention invalid block: {:?}", error_string);
        },
        Ok(_) => panic!("Parser should have returned an error for invalid block"),
    }
}
