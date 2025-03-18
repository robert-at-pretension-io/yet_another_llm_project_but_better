# Meta Programming Language Implementation

This project implements a robust parser and executor for the Meta Programming Language, designed for embedding code, data, and AI interactions directly within structured documents. The implementation now supports XML format for greater interoperability with standard tooling and follows a well-defined language specification with careful handling of edge cases and error conditions.

## Architecture

### 1. Parser (`src/parser/`)

- **XML Parsing**: Parses structured elements with different types and attributes using XML format
- **Block Types**: Handles various block types including code, shell, API, data, template, and control blocks
- **Modifiers**: Processes block attributes that control execution and behavior
- **Variable References**: Extracts references to other blocks using `<meta:reference target="variable-name"/>` XML tags
- **Robust Handling**: Provides graceful parsing for varied syntax patterns, whitespace, indentation, and edge cases

### 2. Executor (`src/executor/`)

- **Modular Architecture**: Uses the Strategy pattern with specialized runners for different block types
- **Block Execution**: Dedicated runners for Python, JavaScript, Shell, conditional, and question blocks
- **State Management**: Centralized state handling via ExecutorState
- **Reference Resolution**: Multi-pass reference processing with XML namespace support
- **Dependency Resolution**: Resolves and executes block dependencies in correct order
- **Caching**: Configurable caching policies via CacheManager
- **Error Handling**: Robust error recovery with fallback mechanisms
- **Backward Compatibility**: Support for existing tests and workflows

### 3. File Watcher (`src/file_watcher/`)

- **File Monitoring**: Watches for changes to specified files in the file system
- **Change Detection**: Identifies file creation, modification, and deletion events
- **Event Notification**: Notifies listeners when watched files are modified

## Block Types (XML Format)

The Meta Programming Language now supports an XML-based format for all block types:

### Communication Blocks
```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question name="user-query">
    What insights can be derived from this data?
  </meta:question>

  <meta:response name="ai-response">
    Based on the data, the key insights are...
  </meta:response>
</meta:document>
```

### Executable Blocks
```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:code language="python" name="data-analysis">
  <![CDATA[
  import pandas as pd
  data = pd.read_csv('data.csv')
  print(data.describe())
  ]]>
  </meta:code>

  <meta:shell name="list-files">
  <![CDATA[
  ls -la
  ]]>
  </meta:shell>

  <meta:api name="get-weather" method="GET">
  <![CDATA[
  https://api.weather.com/forecast?location=NYC
  ]]>
  </meta:api>
</meta:document>
```

### Data Management Blocks
```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="config" format="json">
  <![CDATA[
  {
    "api_key": "abcd1234",
    "endpoint": "https://api.example.com",
    "parameters": {
      "limit": 100,
      "format": "json"
    }
  }
  ]]>
  </meta:data>

  <meta:variable name="greeting">
    Hello, world!
  </meta:variable>

  <meta:secret name="api-key">
    API_KEY_ENV_VAR
  </meta:secret>

  <meta:filename name="data-file">
    data/input.csv
  </meta:filename>

  <meta:memory name="conversation-history">
    Previous conversation content stored across sessions
  </meta:memory>
</meta:document>
```

### Control Blocks
```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:section type="introduction" name="intro-section">
    Section content and nested blocks
  </meta:section>

  <meta:code language="python" name="is-large-dataset">
  <![CDATA[
  import json
  data = json.loads('''<meta:reference target="dataset" />''')
  print("true" if len(data.get("rows", [])) > 100 else "false")
  ]]>
  </meta:code>
  
  <meta:conditional if="is-large-dataset">
    <meta:shell name="large-data-processing">
    <![CDATA[
    echo "Processing large dataset with specialized tools..."
    ]]>
    </meta:shell>
  </meta:conditional>

  <meta:template name="data-processor">
    Template with <meta:reference target="placeholder"/> substitution
  </meta:template>

  <meta:template-invocation name="process-dataset" template="data-processor">
    <meta:param name="placeholder">Value</meta:param>
  </meta:template-invocation>
</meta:document>
```

### Results & Debug Blocks
```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:results for="data-analysis" format="markdown">
    Analysis output content
  </meta:results>

  <meta:error-results for="failed-block">
    Error message details
  </meta:error-results>

  <meta:visualization name="prompt-preview">
    Preview of constructed AI context
  </meta:visualization>

  <meta:preview for="visualization-block">
    Content preview
  </meta:preview>
</meta:document>
```

## Attributes (formerly Modifiers)

Blocks can include a variety of attributes that control their behavior:

### Execution Control
- `cache_result="true|false"` - Enable/disable result caching
- `timeout="30"` - Set execution timeout in seconds
- `retry="3"` - Number of retry attempts on failure
- `fallback="fallback-block"` - Fallback block to use on failure
- `depends="other-block"` - Define execution dependencies
- `async="true|false"` - Enable asynchronous execution

### Display & Formatting
- `format="json|markdown|csv|plain"` - Output format
- `display="inline|block|none"` - Display mode for results
- `trim="true|false"` - Trim whitespace from results
- `max_lines="100"` - Limit displayed lines

### Context Control
- `order="0.5"` - Control block ordering (0.0-1.0)
- `priority="8"` - Set inclusion priority (1-10)
- `weight="0.7"` - Weighting for token budget allocation

## XML Parsing Features

The XML parser offers several advantages:

- **Standard Compliance**: Uses standard XML parsing libraries for robustness
- **CDATA Support**: Properly handles code blocks with special characters using CDATA sections
- **Attribute Processing**: Cleanly processes modifiers as XML attributes
- **Namespace Support**: Uses XML namespaces to avoid conflicts
- **Validation**: Can leverage XML schema validation for structure checking
- **Nested Elements**: Natural handling of hierarchical block structures
- **Tooling Integration**: Works with standard XML tooling for editing and validation

## Usage Example

```rust
use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
use yet_another_llm_project_but_better::executor::runners::{PythonRunner, ShellRunner, QuestionRunner};

fn main() {
    // Create a document with embedded blocks in XML format
    let content = r#"
    <meta:document xmlns:meta="https://example.com/meta-language">
      <meta:data name="user-info" format="json">
      <![CDATA[
      {"name": "Alice", "role": "Developer"}
      ]]>
      </meta:data>

      <meta:code language="python" name="greet-user" depends="user-info">
      <![CDATA[
      import json
      user = json.loads('''<meta:reference target="user-info" />''')
      print(f"Hello, {user['name']}! You are a {user['role']}.")
      ]]>
      </meta:code>
    </meta:document>
    "#;
    
    // Initialize executor with modular architecture
    let mut executor = MetaLanguageExecutor::new();
    
    // Register any custom block runners if needed
    // executor.register_runner(Box::new(CustomRunner));
    
    // Process the entire document (parsing + registering blocks)
    executor.process_document(content).expect("Failed to process document");
    
    // Execute a specific block (dependencies automatically resolved)
    match executor.execute_block("greet-user") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Execution error: {}", e),
    }
    
    // Generate updated document with results
    let updated_doc = executor.update_document().expect("Failed to update document");
    println!("Updated document: {}", updated_doc);
}
```

## Implementation Notes

The implementation prioritizes:

1. **Standard Compatibility**: Uses standard XML parsing for better interoperability
2. **Robustness**: Graceful handling of edge cases and malformed inputs
3. **Flexibility**: Support for varied syntax styles through XML attributes
4. **Error Recovery**: Intelligent fallback mechanisms when parsing fails
5. **Dependency Management**: Careful resolution of block dependencies
6. **Extensibility**: Modular design for adding new block types and features
7. **Deep Reference Resolution**: Multi-pass algorithm for processing nested references

### Reference Resolution

The system implements a sophisticated reference resolution system:

- **Multi-Pass Algorithm**: Resolves references in multiple passes to handle nested references
- **Circular Detection**: Built-in detection and prevention of circular references
- **XML Namespace Support**: Properly handles namespaced elements and references
- **JSON Property Access**: Supports dot-notation for accessing JSON properties
- **Fallback Handling**: Configurable fallback content when references cannot be resolved
- **Deep Nesting**: Supports references of arbitrary depth through iterative resolution

This implementation provides a complete system for parsing and executing embedded code within Meta Language documents using XML format, enabling powerful document-based programming and AI-augmented workflows while maintaining compatibility with standard XML tooling.
