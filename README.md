# Meta Programming Language Implementation

This project implements a robust parser and executor for the Meta Programming Language, designed for embedding code, data, and AI interactions directly within structured documents. The implementation follows a well-defined language specification with careful handling of edge cases and error conditions.

## Architecture

### 1. Parser (`src/parser/`)

- **Block Parsing**: Parses structured blocks with different types and modifiers using a combination of grammar rules and fallback parsing mechanisms
- **Block Types**: Handles various block types including code, shell, API, data, template, and control blocks
- **Modifiers**: Processes block modifiers that control execution and behavior
- **Variable References**: Extracts references to other blocks using `${variable-name}` syntax
- **Robust Handling**: Provides graceful parsing for varied syntax patterns, whitespace, indentation, and edge cases

### 2. Executor (`src/executor/`)

- **Block Execution**: Executes code blocks in various languages, shell commands, and API calls
- **Dependency Resolution**: Resolves dependencies between blocks to ensure correct execution order
- **Variable Substitution**: Processes references to other blocks by inserting their content or results
- **Results Handling**: Generates results blocks with output from executed blocks
- **Error Management**: Creates error results blocks when execution fails, with fallback mechanisms
- **Caching**: Supports result caching based on block modifiers

### 3. File Watcher (`src/file_watcher/`)

- **File Monitoring**: Watches for changes to specified files in the file system
- **Change Detection**: Identifies file creation, modification, and deletion events
- **Event Notification**: Notifies listeners when watched files are modified

## Block Types

The Meta Programming Language supports a wide variety of block types:

### Communication Blocks
```markdown
[question name:user-query]
What insights can be derived from this data?
[/question]

[response name:ai-response]
Based on the data, the key insights are...
[/response]
```

### Executable Blocks
```markdown
[code:python name:data-analysis]
import pandas as pd
data = pd.read_csv('data.csv')
print(data.describe())
[/code:python]

[shell name:list-files]
ls -la
[/shell]

[api name:get-weather method:GET]
https://api.weather.com/forecast?location=NYC
[/api]
```

### Data Management Blocks
```markdown
[data name:config format:json]
{
  "api_key": "abcd1234",
  "endpoint": "https://api.example.com",
  "parameters": {
    "limit": 100,
    "format": "json"
  }
}
[/data]

[variable name:greeting]
Hello, world!
[/variable]

[secret name:api-key]
API_KEY_ENV_VAR
[/secret]

[filename name:data-file]
data/input.csv
[/filename]

[memory name:conversation-history]
Previous conversation content stored across sessions
[/memory]
```

### Control Blocks
```markdown
[section:introduction name:intro-section]
Section content and nested blocks
[/section:introduction]

[conditional if:data.rows > 100]
Conditional content that only appears when the condition is true
[/conditional]

[template name:data-processor]
Template with ${placeholder} substitution
[/template]

[template_invocation name:process-dataset template:data-processor]
Parameter substitution for the template
[/template_invocation]
```

### Results & Debug Blocks
```markdown
[results for:data-analysis format:markdown]
Analysis output content
[/results]

[error_results for:failed-block]
Error message details
[/error_results]

[visualization name:prompt-preview]
Preview of constructed AI context
[/visualization]

[preview for:visualization-block]
Content preview
[/preview]
```

## Block Modifiers

Blocks can include a variety of modifiers that control their behavior:

### Execution Control
- `cache_result:true|false` - Enable/disable result caching
- `timeout:30` - Set execution timeout in seconds
- `retry:3` - Number of retry attempts on failure
- `fallback:fallback-block` - Fallback block to use on failure
- `depends:other-block` - Define execution dependencies
- `async:true|false` - Enable asynchronous execution

### Display & Formatting
- `format:json|markdown|csv|plain` - Output format
- `display:inline|block|none` - Display mode for results
- `trim:true|false` - Trim whitespace from results
- `max_lines:100` - Limit displayed lines

### Context Control
- `order:0.5` - Control block ordering (0.0-1.0)
- `priority:8` - Set inclusion priority (1-10)
- `weight:0.7` - Weighting for token budget allocation

## Parsing Robustness

The parser is designed to handle a wide variety of edge cases:

- **Whitespace Variations**: Handles different indentation styles, line endings, and spacing
- **Block Nesting**: Supports hierarchical block structures
- **Tag Variations**: Processes different closing tag formats and variations
- **Character Escaping**: Correctly handles quoted strings and escape sequences
- **Error Recovery**: Attempts to recover from parsing errors when possible
- **Language Flexibility**: Supports multiple code languages with appropriate syntax

## Usage Example

```rust
use yet_another_llm_project_but_better::parser::parse_document;
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;

fn main() {
    // Parse a document with embedded blocks
    let content = r#"
    [data name:user-info format:json]
    {"name": "Alice", "role": "Developer"}
    [/data]

    [code:python name:greet-user depends:user-info]
    import json
    user = json.loads('${user-info}')
    print(f"Hello, {user['name']}! You are a {user['role']}.")
    [/code:python]
    "#;
    
    let blocks = parse_document(content).expect("Failed to parse document");
    println!("Found {} blocks", blocks.len());
    
    // Execute the blocks with dependency resolution
    let mut executor = MetaLanguageExecutor::new();
    
    // Register blocks
    for block in &blocks {
        if let Some(name) = &block.name {
            executor.blocks.insert(name.clone(), block.clone());
        }
    }
    
    // Execute a block (dependencies automatically resolved)
    match executor.execute_block("greet-user") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Execution error: {}", e),
    }
}
```

## Implementation Notes

The implementation prioritizes:

1. **Robustness**: Graceful handling of edge cases and malformed inputs
2. **Flexibility**: Support for varied syntax styles and whitespace patterns
3. **Error Recovery**: Intelligent fallback mechanisms when parsing fails
4. **Dependency Management**: Careful resolution of block dependencies
5. **Extensibility**: Modular design for adding new block types and features

This implementation provides a complete system for parsing and executing embedded code within Meta Language documents, enabling powerful document-based programming and AI-augmented workflows.