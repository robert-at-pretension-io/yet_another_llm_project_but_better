# Meta Programming Language Implementation

This project implements a parser and executor for the Meta Programming Language, a language designed for embedding code, data, and AI interactions directly within markdown documents.

## Key Components

### 1. Parser (`src/parser/`)

- **Grammar Definition** (`meta_language.pest`): PEG grammar for the language syntax
- **Parser Implementation** (`mod.rs`): Rust code to parse documents into structured blocks

### 2. Executor (`src/executor/`)

- **Block Execution** (`mod.rs`): Logic to execute different block types (Python, JavaScript, Shell, etc.)
- **Variable Substitution**: Replace `${block-name}` references with actual values
- **Dependency Resolution**: Execute blocks in correct order based on dependencies
- **Caching**: Cache block execution results based on modifiers

### 3. Daemon (`src/main.rs`)

- **File Watching**: Monitor document changes and process updates
- **Document Updating**: Insert execution results back into the document

### 4. Test Harness (`src/bin/test_harness.rs`)

- **Document Testing**: Run and verify document execution
- **Output Inspection**: View block outputs and execution results

### 5. Fuzzer (`src/bin/fuzzer.rs`)

- **Grammar Testing**: Apply random mutations to test grammar robustness
- **Edge Case Discovery**: Find interesting parsing edge cases

## Usage

```bash
# Run the daemon to watch a document
cargo run -- /path/to/document.md

# Run the test harness on a document
cargo run --bin test_harness /path/to/test_document.md

# Run the fuzzer to test grammar robustness
cargo run --bin fuzzer
```

## Language Features

The Meta Programming Language supports the following block types:

- **Code Blocks**: Execute code in Python, JavaScript, Rust
- **Shell Blocks**: Run shell commands
- **API Blocks**: Make API requests
- **Data Blocks**: Store structured data
- **Question Blocks**: Define AI interactions
- **Response Blocks**: AI-generated responses
- **Template Blocks**: Reusable document patterns
- **Variable Blocks**: Store named values
- **Visualization Blocks**: Preview content

Blocks support various modifiers:
- `cache_result`: Cache execution results
- `timeout`: Set execution timeout
- `fallback`: Specify fallback block on failure
- `depends`: Declare execution dependencies

## Example

```markdown
[data name:test-data format:json]
{"value": 42}
[/data]

[code:python name:process-data depends:test-data fallback:process-data-fallback]
import json
data = json.loads('''${test-data}''')
print(f"The value is {data['value']}")
[/code:python]

[code:python name:process-data-fallback]
print("Failed to process data")
[/code:python]
```

## Implementation Notes

This implementation follows the language specification in `docs/language_specification.md` and aims to provide a complete and robust system for parsing and executing embedded code within documents.
