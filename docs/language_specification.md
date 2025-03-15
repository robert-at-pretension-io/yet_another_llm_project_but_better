# Meta Programming Language: Technical Documentation

## Overview

The Meta Programming Language is designed for embedding executable code, structured data, and AI interactions within documents. This document explains the technical implementation of the parser, executor, and associated components.

## Core Components

### 1. Parser

The parser is responsible for converting text documents into structured blocks that can be executed or used to construct AI prompts.

#### Block Structure

A block follows this general syntax:
```
[block_type:subtype name:block_name modifier1:value1 modifier2:value2]
block content
[/block_type:subtype]
```

Each block consists of:
- **Block Type**: Primary categorization (e.g., `code`, `data`, `shell`)
- **Subtype** (optional): Further specification (e.g., `code:python`, `section:intro`)
- **Name**: Unique identifier for referencing the block
- **Modifiers**: Key-value pairs controlling behavior or execution
- **Content**: Main content contained within the block

#### Parsing Approach

The parser uses a multi-stage approach for flexibility and robustness:

1. **Grammar Parser**: First attempts to parse using formal grammar rules
2. **Block Parser**: Attempts block-by-block parsing
3. **Manual Parser**: Falls back to manual string extraction techniques
4. **Recovery Parsing**: Attempts to recover from errors and extract as much content as possible

This tiered approach allows the parser to handle a wide variety of syntax variations, whitespace patterns, and even certain malformed structures.

#### Block Type Detection

Block types are detected using these mechanisms:
- Primary block type extraction from opening tag
- Subtype detection after colon (e.g., `code:python`)
- Validation against list of known block types
- Flexible handling of whitespace around type declarations

#### Modifier Handling

Modifiers are parsed from the opening tag with special handling for:
- Basic `key:value` pairs
- Quoted modifiers (e.g., `name:"User Data"`)
- Boolean modifiers (e.g., `cache_result:true`)
- Numerical modifiers (e.g., `timeout:30`, `priority:8`)

#### Whitespace & Indentation

The parser handles various whitespace patterns:
- Diverse indentation styles (spaces, tabs)
- Variable line endings (LF, CRLF)
- Multiline modifier declarations
- Preservation of whitespace in content when meaningful
- Normalization of whitespace when appropriate

#### Character Escaping

Special handling exists for:
- Quoted strings in modifiers
- Escaped characters in content
- JSON embedded within blocks
- HTML and other markup in content

#### Error Recovery

The parser includes mechanisms for:
- Recovering from mismatched closing tags
- Handling malformed block declarations
- Dealing with invalid modifiers
- Parsing blocks with missing attributes
- Capturing syntax errors for debugging

### 2. Block Structure

The `Block` structure is the core data representation:

```rust
pub struct Block {
    // Core attributes
    pub block_type: String,
    pub name: Option<String>,
    pub content: String,
    
    // Additional attributes
    pub modifiers: HashMap<String, String>,
    pub children: Vec<Block>,
    pub parent: Option<Box<Block>>,
}
```

Blocks can be nested hierarchically, forming a tree structure. Parent-child relationships are maintained and can be traversed bidirectionally.

### 3. Executor

The executor is responsible for running executable blocks, resolving dependencies, and managing execution results.

#### Block Execution

The following block types are executable:
- `code:<language>`: Executes code in specified language
- `shell`: Runs shell commands
- `api`: Makes HTTP requests

Execution process:
1. Check block dependencies
2. Resolve variable references
3. Execute content based on block type
4. Capture output in results block
5. Handle errors and fallbacks

#### Dependency Resolution

Dependencies are managed through:
- Explicit `depends` modifiers
- Implicit dependencies via `${block-name}` references
- Topological sorting to determine execution order
- Circular dependency detection
- Fallback mechanisms for dependency failures

#### Variable Substitution

The executor processes variable references:
- Basic substitution: `${block-name}`
- Nested substitution: `${parent.child}`
- Result references: `${block-name.results}`
- Environment variables: `${ENV_VAR}`

#### Caching

Results caching is controlled by:
- `cache_result:true|false` modifier
- Optional time-based expiration
- Cache invalidation on dependency changes
- Context-aware cache key generation

#### Error Handling

Robust error handling includes:
- Execution timeouts based on `timeout` modifier
- Automatic retries based on `retry` modifier
- Fallback to alternative blocks on failure
- Detailed error reporting
- Creation of error results blocks

### 4. File Watcher

The file watcher monitors document changes and triggers re-parsing and execution:

- Uses platform-native file system notifications
- Debounces rapid changes to prevent excessive processing
- Detects file creation, modification, and deletion
- Maintains a list of watched files and directories
- Provides event notifications via channels

## Block Types in Detail

### Communication Blocks

#### Question Block
Represents queries to AI systems:
```markdown
[question name:user-query model:gpt-4 temperature:0.7]
What insights can you provide based on this data?
[/question]
```

#### Response Block
Contains AI-generated responses:
```markdown
[response name:ai-response]
Based on the data, the key trends are...
[/response]
```

### Executable Blocks

#### Code Block
Executes code in various languages:
```markdown
[code:python name:data-analysis cache_result:true]
import pandas as pd
data = pd.read_csv('${data-file}')
print(data.describe())
[/code:python]
```

Supported languages include Python, JavaScript, Bash, and more.

#### Shell Block
Executes system commands:
```markdown
[shell name:list-files timeout:5]
ls -la ${directory}
[/shell]
```

#### API Block
Makes HTTP requests:
```markdown
[api name:get-weather method:GET headers:"Content-Type: application/json"]
https://api.weather.com/forecast?location=${location}
[/api]
```

### Data Management Blocks

#### Data Block
Stores structured data:
```markdown
[data name:config format:json]
{
  "api_key": "abcd1234",
  "endpoint": "https://api.example.com"
}
[/data]
```

#### Variable Block
Defines simple variables:
```markdown
[variable name:greeting]
Hello, ${user-name}!
[/variable]
```

#### Secret Block
References sensitive data from environment:
```markdown
[secret name:api-key]
API_KEY_ENV_VAR
[/secret]
```

#### Filename Block
References external files:
```markdown
[filename name:data-file]
data/input.csv
[/filename]
```

#### Memory Block
Persists data across sessions:
```markdown
[memory name:conversation-history]
Previous interactions stored across sessions
[/memory]
```

### Control Blocks

#### Section Block
Groups related blocks:
```markdown
[section:introduction name:intro-section]
Content and nested blocks
[/section:introduction]
```

#### Conditional Block
Conditionally includes content:
```markdown
[conditional if:data.rows > 100]
This appears only when condition is true
[/conditional]
```

#### Template Block
Defines reusable patterns:
```markdown
[template name:data-processor]
[code:python name:process-${dataset-name}]
import pandas as pd
data = pd.read_csv('${dataset-path}')
[/code:python]
[/template]
```

#### Template Invocation
Uses templates with parameter substitution:
```markdown
[template_invocation name:process-sales template:data-processor]
dataset-name:sales
dataset-path:sales.csv
[/template_invocation]
```

### Results Blocks

#### Results Block
Contains execution output:
```markdown
[results for:data-analysis format:markdown display:block]
Execution output content
[/results]
```

#### Error Results Block
Contains execution errors:
```markdown
[error_results for:failed-block]
Error message and stack trace
[/error_results]
```

### Debugging Blocks

#### Visualization Block
Previews context construction:
```markdown
[visualization name:prompt-preview]
Preview of what will be sent to AI
[/visualization]
```

#### Preview Block
Shows block content previews:
```markdown
[preview for:visualization-block]
Content preview
[/preview]
```

## Modifiers in Detail

### Execution Control Modifiers

| Modifier | Description | Default | Example |
|----------|-------------|---------|---------|
| `cache_result` | Enable/disable result caching | `false` | `cache_result:true` |
| `timeout` | Execution timeout in seconds | None | `timeout:30` |
| `retry` | Number of retry attempts | `0` | `retry:3` |
| `fallback` | Fallback block on failure | None | `fallback:error-handler` |
| `depends` | Execution dependencies | None | `depends:data-block` |
| `async` | Asynchronous execution | `false` | `async:true` |

### Display & Formatting Modifiers

| Modifier | Description | Default | Example |
|----------|-------------|---------|---------|
| `format` | Output format | Auto-detected | `format:json` |
| `display` | Display mode | `block` | `display:inline` |
| `trim` | Trim whitespace | `true` | `trim:false` |
| `max_lines` | Line limit | `0` (unlimited) | `max_lines:100` |

### Context Control Modifiers

| Modifier | Description | Default | Example |
|----------|-------------|---------|---------|
| `order` | Block ordering | Document order | `order:0.5` |
| `priority` | Inclusion priority | `5` | `priority:8` |
| `weight` | Token budget weight | `1.0` | `weight:0.7` |

### Debugging Modifiers

| Modifier | Description | Default | Example |
|----------|-------------|---------|---------|
| `debug` | Enable debug info | `false` | `debug:true` |
| `verbosity` | Debug verbosity | `medium` | `verbosity:high` |

## Implementation Insights

### Parsing Robustness

The parser is designed to handle various edge cases:

1. **Whitespace Tolerance**: The parser handles different whitespace patterns including:
   - Tabs vs. spaces
   - Inconsistent indentation
   - Trailing/leading whitespace
   - Empty lines between blocks

2. **Closing Tag Flexibility**: Various closing tag formats are supported:
   - Full tags: `[/code:python]`
   - Base tags: `[/code]`
   - Type-only tags: `[/block_type]`

3. **Special Character Handling**: The parser correctly processes:
   - Escaped quotes in strings
   - JSON content with nested quotes and brackets
   - Backslashes and escape sequences
   - Special characters in block names and content

4. **Error Recovery**: When encountering parsing errors, the system:
   - Attempts to salvage as much content as possible
   - Provides detailed error information
   - Falls back to simpler parsing strategies
   - Allows the document to be partially processed

### Executor Implementation

The executor contains specialized logic for:

1. **Block Type Execution**:
   - Language-specific execution for code blocks
   - Safe shell command execution
   - HTTP client implementation for API blocks

2. **Dependency Management**:
   - Graph-based dependency resolution
   - Automatic ordering of execution
   - Circular dependency detection
   - Nested dependency handling

3. **Variable Substitution**:
   - Regex-based reference extraction
   - Recursive reference resolution
   - Context-aware substitution
   - Special variable handling

4. **Result Formatting**:
   - Content-type detection
   - Format conversion (JSON, Markdown, etc.)
   - Custom display options
   - Line limiting and trimming

## Testing Strategy

The test suite comprehensively covers:

1. **Parser Tests**:
   - Basic block parsing
   - Complex nested structures
   - Edge cases and error conditions
   - Whitespace and indentation variations
   - Character escaping and special handling

2. **Executor Tests**:
   - Block execution for different types
   - Dependency resolution
   - Variable substitution
   - Error handling and recovery
   - Results generation and formatting

3. **Integration Tests**:
   - Complete document processing
   - Complex workflows with multiple block types
   - Error recovery in full documents
   - Realistic usage scenarios

4. **Robustness Tests**:
   - Malformed input handling
   - Parser recovery mechanisms
   - Edge case detection
   - Stress testing with complex documents

## Usage Guidelines

### Document Structure

1. **Organization**:
   - Group related blocks in sections
   - Use consistent naming conventions
   - Organize dependencies logically
   - Include appropriate fallbacks

2. **Naming**:
   - Use descriptive, unique block names
   - Prefer kebab-case for consistency 
   - Include block type in name when helpful
   - Avoid special characters in names

3. **Modifiers**:
   - Use modifiers consistently
   - Include only necessary modifiers
   - Set appropriate timeouts for long-running blocks
   - Use meaningful priority values

4. **Dependencies**:
   - Explicitly declare dependencies with `depends`
   - Avoid circular dependencies
   - Use variable references carefully
   - Include fallbacks for critical dependencies

### Best Practices

1. **Error Handling**:
   - Always include fallback blocks
   - Check for error conditions
   - Use conditional blocks for error branches
   - Provide meaningful error messages

2. **Performance**:
   - Use caching for expensive operations
   - Minimize unnecessary dependencies
   - Avoid redundant execution
   - Control context size with priorities

3. **Maintainability**:
   - Use templates for repeated patterns
   - Keep blocks focused and single-purpose
   - Document complex blocks with comments
   - Use sections to organize large documents

4. **Security**:
   - Use secret blocks for sensitive data
   - Validate inputs in executable blocks
   - Limit shell command execution
   - Control API access carefully

## Conclusion

The Meta Programming Language implementation provides a robust system for embedding executable code, structured data, and AI interactions within documents. Through its flexible parser, powerful executor, and comprehensive block types, it enables complex workflows while maintaining readability and maintainability.