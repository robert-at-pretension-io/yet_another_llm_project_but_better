# XML Format Tutorial for Meta Programming Language

Welcome to the XML format tutorial for the Meta Programming Language! This guide will help you transition from the original bracket-based syntax to the new XML format, showing how to create, execute, and manage Meta documents using XML.

## Getting Started with XML Format

### Basic Document Structure

Every XML Meta document begins with a standard XML declaration and a root element:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Your blocks go here -->
</meta:document>
```

### Your First XML Meta Document

Let's create a simple document that stores and processes data:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Store data -->
  <meta:data name="numbers" format="json">
  <![CDATA[
  [1, 2, 3, 4, 5]
  ]]>
  </meta:data>
  
  <!-- Process data -->
  <meta:code language="python" name="sum-numbers">
  <![CDATA[
  import json
  numbers = json.loads('''<meta:reference target="numbers" />''')
  total = sum(numbers)
  print(f"The sum is {total}")
  ]]>
  </meta:code>
  
  <!-- Display results -->
  <meta:results for="sum-numbers" format="plain">
  The sum is 15
  </meta:results>
</meta:document>
```

## Block Types and Examples

### Communication Blocks

#### Question Block
Ask AI systems for help:

```xml
<meta:question model="gpt-4" temperature="0.7" name="data-query">
Given the following sales data, what are the key trends to note?
<meta:reference target="sales-data" />
</meta:question>
```

#### Response Block
Display AI-generated responses:

```xml
<meta:response name="analysis-response" for="data-query">
Based on the sales data, the key trends are:
1. Seasonal peaks in Q4
2. Year-over-year growth of 15%
3. Product line A outperforming others
</meta:response>
```

### Executable Blocks

#### Code Block
Execute code in various languages:

```xml
<meta:code language="python" name="visualize-data" cache_result="true">
<![CDATA[
import matplotlib.pyplot as plt
import pandas as pd

data = pd.read_csv('''<meta:reference target="data-file" />''')
plt.figure(figsize=(10, 6))
plt.plot(data['date'], data['value'])
plt.title('Data Visualization')
plt.savefig('output.png')
print("Visualization created")
]]>
</meta:code>
```

#### Shell Block
Run shell commands:

```xml
<meta:shell name="check-environment" timeout="5">
<![CDATA[
uname -a
python --version
pip list | grep pandas
]]>
</meta:shell>
```

#### API Block
Make API requests:

```xml
<meta:api name="fetch-weather" method="GET" cache_result="true">
https://api.weather.com/forecast?location=<meta:reference target="location" />&units=metric
</meta:api>
```

### Data Management

#### Data Block
Store structured data:

```xml
<meta:data name="user-preferences" format="json">
<![CDATA[
{
  "theme": "dark",
  "fontSize": 14,
  "enableNotifications": true
}
]]>
</meta:data>
```

#### Variable Block
Define simple variables:

```xml
<meta:variable name="api-endpoint">
https://api.example.com/v2
</meta:variable>
```

#### Secret Block
Reference environment variables:

```xml
<meta:secret name="api-key">
API_KEY_ENV_VAR
</meta:secret>
```

## Control Blocks

### Section Block
Group related blocks:

```xml
<meta:section type="analysis" name="sales-report">
  <meta:data name="sales-data" format="json">
  <![CDATA[
  {"sales": 1000, "period": "Q1"}
  ]]>
  </meta:data>
  
  <meta:code language="python" name="analyze-sales">
  <![CDATA[
  import json
  data = json.loads('${sales-data}')
  print(f"Sales: {data['sales']} in {data['period']}")
  ]]>
  </meta:code>
</meta:section>
```

### Conditional Block
Execute content only if a condition block returns "true":

```xml
<!-- Define a condition -->
<meta:code language="python" name="is-large-dataset">
<![CDATA[
import json
data = json.loads('''<meta:reference target="dataset" />''')
print("true" if len(data["rows"]) > 1000 else "false")
]]>
</meta:code>

<!-- Use the condition -->
<meta:conditional if="is-large-dataset">
  <meta:code language="python" name="big-data-process">
  <![CDATA[
  # Code for large datasets
  print("Processing large dataset...")
  ]]>
  </meta:code>
</meta:conditional>
```

Conditional blocks execute their content only when the referenced block returns "true", "1", or "yes" (case insensitive). You can use any block that produces output as a condition, including:

- Python code blocks that print "true" or "false"
- Shell commands that echo "true" or "false"  
- LLM question blocks that return "true" or "false"

This allows for powerful decision-making capabilities, including:
- Data-driven conditions based on analysis results
- Environment-based conditions (development vs. production)
- User-role-based conditions (admin vs. regular user)
- "Smart" conditions based on LLM reasoning

### Template Block
Create reusable patterns:

```xml
<meta:template name="data-processor">
  <meta:code language="python" name="process-<meta:reference target="dataset-name" />">
  <![CDATA[
  import pandas as pd
  data = pd.read_csv('''<meta:reference target="dataset-path" />''')
  processed = data.describe()
  print(processed)
  ]]>
  </meta:code>
</meta:template>
```

### Template Invocation
Use templates with custom parameters:

```xml
<meta:template-invocation name="process-sales" template="data-processor">
  <meta:param name="dataset-name">sales</meta:param>
  <meta:param name="dataset-path">sales.csv</meta:param>
</meta:template-invocation>
```

## Advanced Features

### Variable References
Reference other blocks using XML tags:

```xml
<meta:code language="python" name="process-user">
<![CDATA[
import json
prefs = json.loads('''<meta:reference target="user-preferences" />''')
print(f"Using {prefs['theme']} theme with {prefs['fontSize']}px font")
]]>
</meta:code>
```

You can add modifiers to the reference tag:
```xml
<meta:reference target="data-block" format="json" />
<meta:reference target="code-block" include_code="true" include_results="true" />
<meta:reference target="missing-data" fallback="Data not available" />
```

### Nested Blocks
Create complex workflows with nested blocks:

```xml
<meta:section type="analysis" name="complete-analysis">
  <meta:data name="input-data" format="json">
  <![CDATA[
  {"values": [1, 2, 3, 4, 5]}
  ]]>
  </meta:data>
  
  <meta:code language="python" name="analyze">
  <![CDATA[
  import json
  data = json.loads('''<meta:reference target="input-data" />''')
  result = sum(data['values'])
  print(f"Sum: {result}")
  ]]>
  </meta:code>
  
  <meta:conditional if="result > 10">
    <meta:shell name="notify">
    <![CDATA[
    echo "Result exceeds threshold: <meta:reference target="analyze.results" />"
    ]]>
    </meta:shell>
  </meta:conditional>
</meta:section>
```

### Deep Nested Reference Resolution

The Meta Programming Environment includes robust support for deeply nested references, which allows for complex data transformations and workflows.

#### Multi-Level References

References can be nested within other blocks' content and are resolved recursively:

```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Level 1: Basic data -->
  <meta:data name="user" format="json">
  <![CDATA[
  {"name": "Alice", "role": "Admin"}
  ]]>
  </meta:data>
  
  <!-- Level 2: Reference to user data -->
  <meta:variable name="greeting">
  Hello, <meta:reference target="user.name" />, you are a <meta:reference target="user.role" />!
  </meta:variable>
  
  <!-- Level 3: Reference to greeting that contains references -->
  <meta:code language="python" name="welcome-message">
  <![CDATA[
  message = """<meta:reference target="greeting" />"""
  print(f"Welcome message: {message}")
  ]]>
  </meta:code>
</meta:document>
```

In this example:
1. The `greeting` variable references fields from the `user` data block
2. The `welcome-message` code block references the `greeting` variable
3. The reference resolution happens inside-out, so all nested references are resolved

#### Testing Deep Nested References

To create robust tests for deep nested references:

```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Base data -->
  <meta:data name="config" format="json">
  <![CDATA[
  {
    "database": {
      "host": "db.example.com",
      "port": 5432,
      "credentials": "CONFIG_DB_CREDS"
    }
  }
  ]]>
  </meta:data>
  
  <!-- Secret substitution -->
  <meta:secret name="database-credentials">
  DATABASE_CREDENTIALS_ENV
  </meta:secret>
  
  <!-- Level 1: Template with reference -->
  <meta:template name="connection-string">
  postgresql://<meta:reference target="database-credentials" />@<meta:reference target="config.database.host" />:<meta:reference target="config.database.port" />
  </meta:template>
  
  <!-- Level 2: Code with template reference -->
  <meta:code language="python" name="db-connect">
  <![CDATA[
  connection_string = '''<meta:reference target="connection-string" />'''
  print(f"Connecting to: {connection_string}")
  ]]>
  </meta:code>
  
  <!-- Level 3: Shell with nested code reference -->
  <meta:shell name="log-connection">
  <![CDATA[
  echo "About to connect with: <meta:reference target="db-connect" />"
  ]]>
  </meta:shell>
  
  <!-- Test assertions -->
  <meta:code language="python" name="verify-resolution" test="true">
  <![CDATA[
  output = '''<meta:reference target="log-connection" />'''
  # The output should have fully resolved all nested references
  assert "Connecting to: postgresql://" in output
  assert "db.example.com:5432" in output
  print("All nested references correctly resolved.")
  ]]>
  </meta:code>
</meta:document>
```

#### Multi-Pass Resolution Algorithm

The Meta Environment uses a sophisticated multi-pass resolution algorithm to handle deeply nested references:

1. **First Pass**: Resolves direct references to data blocks and variables
2. **Second Pass**: Resolves references to blocks that contain references to other blocks
3. **Final Pass**: Performs a final resolution to catch any remaining references

This approach ensures that even deeply nested references are correctly resolved before execution.

#### Testing Circular References

The system also includes detection for circular references:

```xml
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Circular reference test -->
  <meta:variable name="var-a">
  Value A with reference to B: <meta:reference target="var-b" />
  </meta:variable>
  
  <meta:variable name="var-b">
  Value B with reference to C: <meta:reference target="var-c" />
  </meta:variable>
  
  <meta:variable name="var-c">
  Value C with reference back to A: <meta:reference target="var-a" />
  </meta:variable>
  
  <!-- Test execution will detect circular reference -->
  <meta:code language="python" name="test-circular">
  <![CDATA[
  try:
      # This should throw a CircularDependency error
      value = '''<meta:reference target="var-a" />'''
      print("ERROR: Failed to detect circular reference")
  except Exception as e:
      print(f"Correctly detected circular reference: {e}")
  ]]>
  </meta:code>
</meta:document>
```

#### XML Namespace Resolution

The system supports XML namespaces in reference resolution, allowing for more complex document structures:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language" 
               xmlns:data="https://example.com/data"
               xmlns:app="https://example.com/app">
  <!-- Data in different namespace -->
  <meta:data name="settings" format="xml">
  <![CDATA[
  <data:settings>
    <data:theme>dark</data:theme>
    <data:language>en-US</data:language>
  </data:settings>
  ]]>
  </meta:data>
  
  <!-- Reference resolving across namespaces -->
  <meta:code language="python" name="xml-processor">
  <![CDATA[
  import xml.etree.ElementTree as ET
  from io import StringIO
  
  # Parse settings XML
  settings_xml = '''<meta:reference target="settings" />'''
  root = ET.parse(StringIO(settings_xml)).getroot()
  
  # Extract values with namespace awareness
  ns = {'data': 'https://example.com/data'}
  theme = root.find('.//data:theme', ns).text
  language = root.find('.//data:language', ns).text
  
  print(f"Theme: {theme}, Language: {language}")
  ]]>
  </meta:code>
</meta:document>
```

When running extensive tests of deep nested reference resolution, the Meta Environment provides detailed logs (when debug mode is enabled) showing the resolution process and any issues detected.

### Error Handling
Specify fallback blocks for error recovery:

```xml
<meta:code language="python" name="risky-operation" fallback="safe-fallback">
<![CDATA[
import requests
response = requests.get('${api-endpoint}/data')
data = response.json()
print(f"Received {len(data)} items")
]]>
</meta:code>

<meta:code language="python" name="safe-fallback">
<![CDATA[
print("Using fallback: No data available")
]]>
</meta:code>
```

## Tips and Best Practices

### CDATA Usage
Always use CDATA sections for code and structured data:

```xml
<meta:code language="python" name="example">
<![CDATA[
# Your Python code here
if x < 10:
    print(f"Value: {x}")
]]>
</meta:code>
```

### Proper Indentation
Maintain consistent indentation for readability:

```xml
<meta:section type="analysis" name="report">
  <meta:data name="dataset" format="json">
  <![CDATA[
  {
    "values": [1, 2, 3]
  }
  ]]>
  </meta:data>
  
  <meta:code language="python" name="process">
  <![CDATA[
  import json
  data = json.loads('${dataset}')
  print(data['values'])
  ]]>
  </meta:code>
</meta:section>
```

### Attribute Formatting
Use consistent attribute formatting:

```xml
<meta:code 
  language="python"
  name="complex-processing"
  timeout="30"
  cache_result="true"
  fallback="simple-processing"
>
<![CDATA[
# Complex processing code
]]>
</meta:code>
```

### Document Organization
Group related blocks in sections:

```xml
<meta:section type="data-ingestion" name="input-processing">
  <!-- Data loading blocks -->
</meta:section>

<meta:section type="analysis" name="data-analysis">
  <!-- Analysis blocks -->
</meta:section>

<meta:section type="visualization" name="data-visualization">
  <!-- Visualization blocks -->
</meta:section>
```

## Converting Between Formats

### Meta to XML Conversion
Use the conversion utility to convert existing Meta documents:

```bash
meta-convert to-xml document.meta document.xml
```

### XML to Meta Conversion
Convert XML documents back to Meta format:

```bash
meta-convert to-meta document.xml document.meta
```

### Batch Conversion
Convert multiple files at once:

```bash
meta-convert batch-convert ./documents
```

## Validation

### Using XML Schema
Validate your XML documents against the Meta schema:

```bash
xmllint --schema meta-language.xsd document.xml --noout
```

## Advanced Topics

### Modular Executor Architecture

The Meta Processing Environment uses a modular executor architecture that makes it easy to extend and customize:

1. **BlockRunner Trait**: All execution is handled by implementations of the BlockRunner trait
   ```rust
   pub trait BlockRunner: Send + Sync {
       fn can_execute(&self, block: &Block) -> bool;
       fn execute(&self, block_name: &str, block: &Block, state: &mut ExecutorState) -> Result<String, ExecutorError>;
   }
   ```

2. **Runner Registry**: A central registry manages all available runners
   ```rust
   // Automatically registers standard runners
   let mut executor = MetaLanguageExecutor::new();
   
   // Register a custom runner
   executor.register_runner(Box::new(MyCustomRunner));
   ```

3. **Customizing Behavior**: Create custom runners to handle specialized block types
   ```rust
   pub struct CustomDataRunner;
   
   impl BlockRunner for CustomDataRunner {
       fn can_execute(&self, block: &Block) -> bool {
           block.block_type == "data:special"
       }
       
       fn execute(&self, name: &str, block: &Block, state: &mut ExecutorState) -> Result<String, ExecutorError> {
           // Custom implementation here
       }
   }
   ```

4. **State Management**: All runners share access to the central ExecutorState
   ```rust
   // Access shared state in a runner
   fn execute(&self, name: &str, block: &Block, state: &mut ExecutorState) -> Result<String, ExecutorError> {
       // Access blocks
       let blocks = &state.blocks;
       
       // Access outputs
       let outputs = &state.outputs;
       
       // Store results
       state.store_block_output(name, result.clone());
   }
   ```

### Caching System

The Meta Processing Environment includes a configurable caching system:

1. **Cache Control**: Set caching behavior with the `cache_result` attribute
   ```xml
   <meta:code language="python" name="expensive-calculation" cache_result="true">
   <!-- Code that takes a long time to run -->
   </meta:code>
   ```

2. **Cache Timeout**: Control how long results are cached with the `timeout` attribute
   ```xml
   <meta:api name="weather-data" method="GET" cache_result="true" timeout="3600">
   https://api.weather.com/forecast
   </meta:api>
   ```

3. **Cache Policies**: Different block types have different default caching policies:
   - Data blocks: Always cached
   - API blocks: Cached with explicit `cache_result="true"`
   - Python code blocks: Cached only with explicit `cache_result="true"`
   - Shell blocks: Not cached by default

## Conclusion

The XML format provides all the features of the original Meta format with the added benefits of standardization, validation, and industry-standard tooling. Both formats are fully compatible and can be used interchangeably in the Meta Processing Environment.

The new modular executor architecture allows for greater flexibility, extensibility, and maintainability, making it easy to add new block types and customize behavior.

As you develop with the XML format and modular architecture, remember that the concepts and functionality remain the same—only the implementation has been enhanced to provide better organization and extensibility.

Start creating your XML Meta documents today, and enjoy the enhanced capabilities they provide!