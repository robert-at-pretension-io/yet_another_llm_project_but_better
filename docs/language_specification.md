# Meta Programming Language: XML Format Specification

## Overview

The Meta Programming Language now supports two syntaxes: the original bracket-based syntax and an XML-based syntax. This document specifies the XML format, which provides the same functionality with the benefits of standard XML tooling, validation, and parsing.

## Core Concepts

### Document Structure

An XML Meta document follows this structure:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Blocks go here -->
  <meta:code language="python" name="example">
  <![CDATA[
  print("Hello, world!")
  ]]>
  </meta:code>
  
  <meta:data name="user-info" format="json">
  <![CDATA[
  {"name": "Alice", "role": "Developer"}
  ]]>
  </meta:data>
</meta:document>
```

Every document:
- Uses the `meta` namespace prefix
- Includes blocks as child elements of the root `meta:document` element
- Represents block attributes as XML attributes
- Contains block content as element text or CDATA sections

### Block Structure

Each block consists of:
- **Element Name**: Corresponds to the block type (e.g., `meta:code`, `meta:data`)
- **Attributes**: Block name and modifiers as XML attributes
- **Content**: Main content contained within the element
- **Children**: For blocks that can contain other blocks

### Namespaces

All elements use the `meta` namespace to avoid conflicts with other XML vocabularies:
```
xmlns:meta="https://example.com/meta-language"
```

### CDATA Sections

For blocks containing code or structured data that might include special characters, CDATA sections are used:

```xml
<meta:code language="python" name="example">
<![CDATA[
if x < 10:
    print(f"Value is {x}")
]]>
</meta:code>
```

## Block Types in Detail

### Communication Blocks

#### Question Block
Represents queries to AI systems:
```xml
<meta:question name="user-query" model="gpt-4" temperature="0.7">
What insights can you provide based on this data?
</meta:question>
```

#### Response Block
Contains AI-generated responses:
```xml
<meta:response name="ai-response">
Based on the data, the key trends are...
</meta:response>
```

### Executable Blocks

#### Code Block
Executes code in various languages:
```xml
<meta:code language="python" name="data-analysis" cache_result="true">
<![CDATA[
import pandas as pd
data = pd.read_csv('${data-file}')
print(data.describe())
]]>
</meta:code>
```

Supported languages include Python, JavaScript, Bash, and more.

#### Shell Block
Executes system commands:
```xml
<meta:shell name="list-files" timeout="5">
<![CDATA[
ls -la ${directory}
]]>
</meta:shell>
```

#### API Block
Makes HTTP requests:
```xml
<meta:api name="get-weather" method="GET" headers="Content-Type: application/json">
https://api.weather.com/forecast?location=${location}
</meta:api>
```

### Data Management Blocks

#### Data Block
Stores structured data:
```xml
<meta:data name="config" format="json">
<![CDATA[
{
  "api_key": "abcd1234",
  "endpoint": "https://api.example.com"
}
]]>
</meta:data>
```

#### Variable Block
Defines simple variables:
```xml
<meta:variable name="greeting">
Hello, ${user-name}!
</meta:variable>
```

#### Secret Block
References sensitive data from environment:
```xml
<meta:secret name="api-key">
API_KEY_ENV_VAR
</meta:secret>
```

#### Filename Block
References external files:
```xml
<meta:filename name="data-file">
data/input.csv
</meta:filename>
```

#### Memory Block
Persists data across sessions:
```xml
<meta:memory name="conversation-history">
Previous interactions stored across sessions
</meta:memory>
```

### Control Blocks

#### Section Block
Groups related blocks:
```xml
<meta:section type="introduction" name="intro-section">
  <meta:data name="section-data" format="json">
  <![CDATA[
  {"title": "Introduction"}
  ]]>
  </meta:data>
  
  <meta:code language="python" name="intro-code">
  <![CDATA[
  print("Welcome to the introduction")
  ]]>
  </meta:code>
</meta:section>
```

#### Conditional Block
Conditionally includes content:
```xml
<meta:conditional if="data.rows > 100">
  <meta:code language="python" name="large-data-processing">
  <![CDATA[
  process_large_dataset(data)
  ]]>
  </meta:code>
</meta:conditional>
```

#### Template Block
Defines reusable patterns:
```xml
<meta:template name="data-processor">
  <meta:code language="python" name="process-${dataset-name}">
  <![CDATA[
  import pandas as pd
  data = pd.read_csv('${dataset-path}')
  ]]>
  </meta:code>
</meta:template>
```

#### Template Invocation
Uses templates with parameter substitution:
```xml
<meta:template-invocation name="process-sales" template="data-processor">
  <meta:param name="dataset-name">sales</meta:param>
  <meta:param name="dataset-path">sales.csv</meta:param>
</meta:template-invocation>
```

### Results Blocks

#### Results Block
Contains execution output:
```xml
<meta:results for="data-analysis" format="markdown" display="block">
Execution output content
</meta:results>
```

#### Error Results Block
Contains execution errors:
```xml
<meta:error-results for="failed-block">
Error message and stack trace
</meta:error-results>
```

### Debugging Blocks

#### Visualization Block
Previews context construction:
```xml
<meta:visualization name="prompt-preview">
  <meta:question model="gpt-4" name="sample-query">
  How would you summarize this data?
  </meta:question>
</meta:visualization>
```

#### Preview Block
Shows block content previews:
```xml
<meta:preview for="visualization-block">
Content preview
</meta:preview>
```

## Attributes in Detail

### Common Attributes

All blocks can have these attributes:

| Attribute | Description | Example |
|-----------|-------------|---------|
| `name` | Block identifier | `name="data-loader"` |
| `cache_result` | Enable/disable result caching | `cache_result="true"` |
| `timeout` | Execution timeout in seconds | `timeout="30"` |
| `retry` | Number of retry attempts | `retry="3"` |
| `fallback` | Fallback block on failure | `fallback="error-handler"` |
| `depends` | Execution dependencies | `depends="data-block"` |
| `async` | Asynchronous execution | `async="true"` |

### Display & Formatting Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `format` | Output format | `format="json"` |
| `display` | Display mode | `display="inline"` |
| `trim` | Trim whitespace | `trim="false"` |
| `max_lines` | Line limit | `max_lines="100"` |

### Context Control Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `order` | Block ordering | `order="0.5"` |
| `priority` | Inclusion priority | `priority="8"` |
| `weight` | Token budget weight | `weight="0.7"` |

### Debugging Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `debug` | Enable debug info | `debug="true"` |
| `verbosity` | Debug verbosity | `verbosity="high"` |

## Variable References

Variable references follow the same syntax as in the bracket-based format:

```xml
<meta:code language="python" name="process-data">
<![CDATA[
import json
user_data = json.loads('${user-info}')
print(f"Hello, {user_data['name']}!")
]]>
</meta:code>
```

References can point to:
- Block content: `${block-name}`
- Results: `${block-name.results}`
- Environment variables: `${ENV_VAR}`

## XML Schema

An XML Schema Definition (XSD) is available for validation:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" 
           xmlns:meta="https://example.com/meta-language"
           targetNamespace="https://example.com/meta-language"
           elementFormDefault="qualified">
  <!-- Schema elements here -->
</xs:schema>
```

The complete schema is available in the `meta-language.xsd` file.

## Implementation Notes

### Parsing and Validation

The Meta Processing Environment supports both formats transparently:
- Automatic format detection
- Format-specific parsing
- Schema validation for XML format
- Equivalent execution regardless of format

### Performance Considerations

XML parsing uses the high-performance `quick-xml` library, providing:
- Fast parsing and serialization
- Low memory usage
- Streaming capabilities for large documents

### Special Character Handling

The XML format handles special characters through:
- CDATA sections for code and structured data
- XML entity encoding for text content
- XML namespaces to avoid conflicts

### Format Conversion

Bidirectional conversion between formats is supported:
- Convert bracket format to XML: `meta-convert to-xml input.meta output.xml`
- Convert XML format to brackets: `meta-convert to-meta input.xml output.meta`

## Best Practices

### Document Organization

1. **Structure**:
   - Use sections to group related blocks
   - Place dependencies before dependent blocks
   - Use meaningful element names and attributes

2. **Naming Conventions**:
   - Use kebab-case for `name` attributes
   - Use descriptive names that indicate purpose
   - Maintain consistent naming patterns

3. **Content Management**:
   - Use CDATA for code, JSON, and special characters
   - Keep blocks focused and single-purpose
   - Use templates for repeated patterns

4. **Error Handling**:
   - Always specify fallback blocks
   - Provide descriptive error messages
   - Use conditional blocks for error branches

## Conclusion

The XML format for the Meta Programming Language provides all the functionality of the original bracket-based syntax with added benefits of standard tooling, improved validation, and better error reporting. Both formats are fully supported, allowing seamless migration and interoperability.