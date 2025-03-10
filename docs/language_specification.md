# Technical Specification for the Meta Programming Language

## 1. Document Structure
A document consists of clearly delimited blocks organized hierarchically into optional sections.

### Block Syntax
```markdown
[block_type name:block_name modifiers]
block content
[/block_type]
```
- `block_type`: Required. Specifies functionality (e.g., `code`, `question`, `data`).
- `name`: Required. A unique identifier for block referencing. **Must be unique across the document.**
- `modifiers` *(optional)*: Controls behavior and execution specifics.

## 2. Core Block Types

### Communication Blocks
- `question`: Triggers AI interaction.
- `response`: AI-generated content (auto-inserted by daemon).
- `comment`: Notes ignored by AI context unless specified.

### Executable Blocks
- `code:<language>`: Executes specified code.
- `shell`: Executes system-level commands.
- `api`: Performs external API requests.

### Data Management Blocks
- `data`: Stores structured data with an optional schema.
- `variable`: Stores named, reusable values.
- `secret`: Loads sensitive data from environment variables.
- `filename`: Includes external file content into the context, path can be relative or absolute.

### Results Blocks
- `results`: Contains execution outputs from executable blocks (auto-inserted by daemon).
- `error_results`: Displays execution errors (auto-inserted by daemon when execution fails).

### Control Blocks
- `template`: Defines reusable block structures with placeholders.
- `memory`: Stores persistent context across sessions.
- `conditional`: Conditionally includes blocks based on expressions.
- `error`: Explicitly represents execution errors.

### Debugging Blocks
- `debug`: Toggles verbose logging and execution metadata.
- `visualization`: Generates preview of constructed contexts.
- `preview`: Auto-generated prompt previews within visualization contexts.

## 3. Modifiers

### Execution Control Modifiers
- `cache_result:true|false` (default `false`)
- `timeout:<seconds>` (default: no timeout)
- `retry:<int>` (default: `0`)
- `fallback:block_name` (mandatory for executable blocks)
- `async:true|false` (default `false`)

### Context Management Modifiers
- `always_include:true|false` (default: named blocks `false`, unnamed blocks `true`)
- `priority:[1-10]` (default: `5`)
- `order:[0.0-1.0]` (default: document order)
- `weight:[0.0-1.0]` (default: `1.0`)
- `summarize:brief|semantic|tabular` (optional)

### Results Modifiers
- `display:inline|block|none` (default: `block`)
- `format:plain|json|csv|markdown` (default: based on output content)
- `trim:true|false` (default: `true`)
- `max_lines:<int>` (default: `0` for unlimited)

### Debugging Modifiers
- `debug:true|false` (default: `false`)
- `verbosity:low|medium|high|full` (default: `medium`)

## 4. Block Dependencies
Dependencies explicitly declared using modifiers:
- `depends:<block_name>` (required execution)
- `requires:<block_name>` (block inclusion without execution requirement)

### Dependency Resolution Rules
- Explicit modifiers always supersede implicit references.
- Circular dependencies halt execution, generating explicit `error` blocks.

## 5. Context Building
When encountering a `question` block, the daemon:
1. Initializes empty context.
2. Resolves explicit dependencies (`depends`, `requires`).
3. Executes dependent executable blocks (`code`, `shell`, `api`) and captures outputs.
4. Includes implicit dependencies from `${block_name}` references.
5. Includes `always_include` blocks.
6. Constructs final ordered prompt for AI.

### Context Ordering
Blocks are ordered as follows:
1. Blocks with `order` modifier (ascending).
2. Blocks with higher `priority` (descending).
3. Document natural ordering for equal priority and order.

### Context Pruning
Upon exceeding token limits:
- Blocks with lower priority values pruned first.
- Summarization modifiers applied (`summarize:brief|semantic|tabular`).
- Blocks with highest priority (`10`) preserved at all costs.

## 6. Execution Results
When executing blocks:

1. Automatic results inclusion: 
   - The daemon automatically inserts a `results` block after each executed block.
   - The `results` block contains the stdout/stderr of the executed block.

2. Results block syntax:
   ```markdown
   [results for:block_name format:format_type display:display_type]
   execution output content
   [/results]
   ```

3. Error results:
   - If execution fails, an `error_results` block is inserted instead.
   ```markdown
   [error_results for:block_name]
   error message and stack trace
   [/error_results]
   ```

4. Results processing:
   - Results can be referenced using `${block_name.results}` in subsequent blocks.
   - Results are formatted according to the `format` modifier.
   - Display can be controlled with the `display` modifier.
   - Large outputs can be truncated using the `max_lines` modifier.

## 7. Template Expansion
- Template blocks define placeholders (`${placeholder}` syntax).
- Placeholders substituted during template invocation.

### Template Syntax
```markdown
[template name:template_name]
block definitions with placeholders
[/template]

[@template_name placeholder1:"value" placeholder2:"value"]
[/@template_name]
```

## 8. Mandatory Fallbacks
All executable blocks must specify a fallback block explicitly:
- Fallback blocks named `<original-block-name>-fallback`.
- Daemon auto-generates and inserts if missing:

Example:
```markdown
[code:python name:fetch-data fallback:fetch-data-fallback]
execute_critical_operation()
[/code:python]

[code:python name:fetch-data-fallback]
handle_error_gracefully()
[/code:python]
```

## 9. Error Handling
Explicit error blocks halt execution, require manual resolution:
```markdown
[error type:error_type]
Descriptive error message.
[/error]
```

### Predefined Errors
- `namespace_conflict`: Duplicate block names.
- `circular_dependency`: Circular block dependency detected.
- `execution_failure`: Execution of block failed with no fallback.

## 10. Debugging and Visualization
Debugging blocks clearly provide:
- Dependency graphs.
- Execution timestamps and logs.
- Context pruning logs.
- Prompt previews.

Visualization wraps question blocks to produce explicit previews:
```markdown
[visualization]
  [question debug:true]
  What is the analysis?
  [/question]

  [preview]
  (The preview block contains the fully parsed and finalized prompt exactly as it will be sent to the LLM for execution.)
  [/preview]
[/visualization]
```

## 11. Version Control
- Automatic commits on document changes.
- Each commit includes block-level change metadata.
- Supports rollback and branching for version management.

## 12. Security Considerations
- Secrets loaded from environment variables.
- Secret blocks never included in AI contexts.
- Permission modifiers can restrict access at the block-level.

## 13. Environment
- Document daemon continuously monitors, resolves dependencies, and executes as per the specification.
- On daemon restart, pending question blocks resolved automatically.

---

This specification provides precise and unambiguous guidance, ensuring exact implementation without requiring external clarification.