# XML Format Examples

This document provides examples of Meta Programming Language documents in XML format. The examples demonstrate the features of the language and how they interact with the modular executor architecture.

## Basic Example

```

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="test-data" format="json">
  <![CDATA[
  {"value": 42, "message": "Hello, world!"}
  ]]>
  </meta:data>
  
  <meta:code language="python" name="process-data" auto_execute="true">
  <![CDATA[
  import json
  data = '''{"value": 42, "message": "Hello, world!"}'''
  parsed = json.loads(data)
  print(f"The value is {parsed['value']} and the message is '{parsed['message']}'")
  ]]>
  </meta:code>
  
  <meta:shell name="list-files" auto_execute="true">
  <![CDATA[
  ls -la
  ]]>
  </meta:shell>
</meta:document>
```

## Variable References

```

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="numbers" format="json">
  <![CDATA[
  [1, 2, 3, 4, 5]
  ]]>
  </meta:data>
  
  <meta:code language="python" name="sum-numbers">
  <![CDATA[
  import json
  numbers = json.loads('''<meta:reference target="numbers" />''')
  total = sum(numbers)
  print(f"The sum is {total}")
  ]]>
  </meta:code>
</meta:document>
```

## Nested Sections

```

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:section type="analysis" name="sales-report">
    <meta:data name="sales-data" format="json">
    <![CDATA[
    {"sales": 1000}
    ]]>
    </meta:data>
    
    <meta:code language="python" name="analyze-sales">
    <![CDATA[
    import json
    data = json.loads('''<meta:reference target="sales-data" />''')
    print(f"Sales: {data['sales']}")
    ]]>
    </meta:code>
  </meta:section>
</meta:document>
```

## Questions and Responses

```

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:question model="gpt-4">
  Explain recursion clearly.
  </meta:question>
  
  <meta:response>
  Recursion is a programming concept where a function calls itself to solve smaller instances of the same problem. Like a Russian nesting doll, each call works with a simpler version until reaching a "base case" that can be solved directly.
  
  For example, calculating factorial:
  - 5! = 5 × 4!
  - 4! = 4 × 3!
  - 3! = 3 × 2!
  - 2! = 2 × 1!
  - 1! = 1 (base case)
  
  Each step depends on a simpler version until we hit the simplest case.
  </meta:response>
</meta:document>
```

## Templates and Invocations

```

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:template name="data-insights" model="gpt-4" temperature="0.3">
    <meta:question model="<meta:reference target="model" />" temperature="<meta:reference target="temperature" />">
    Analyze this dataset: <meta:reference target="dataset" />
    </meta:question>
  </meta:template>
  
  <meta:template-invocation name="insights-invocation" template="data-insights">
    <meta:param name="dataset"><meta:reference target="sales-data" /></meta:param>
  </meta:template-invocation>
</meta:document>
```

## Modular Executor Example

The following example demonstrates how the modular executor architecture handles different block types using specialized runners:

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Data block - Processed by basic interpreter -->
  <meta:data name="config" format="json">
  <![CDATA[
  {
    "cache_enabled": true,
    "verbosity": "high",
    "threshold": 75
  }
  ]]>
  </meta:data>
  
  <!-- Python code block - Processed by PythonRunner -->
  <meta:code language="python" name="data-processor" cache_result="true">
  <![CDATA[
  import json
  import numpy as np
  
  # Load configuration
  config = json.loads('''<meta:reference target="config" />''')
  
  # Generate sample data
  data = np.random.normal(50, 15, 100).tolist()
  
  # Apply threshold from config
  threshold = config["threshold"]
  filtered = [x for x in data if x > threshold]
  
  # Return statistics
  stats = {
    "original_count": len(data),
    "filtered_count": len(filtered),
    "filtered_percentage": (len(filtered) / len(data)) * 100
  }
  
  print(json.dumps(stats, indent=2))
  ]]>
  </meta:code>
  
  <!-- Shell block - Processed by ShellRunner -->
  <meta:shell name="system-info">
  <![CDATA[
  echo "System Information:"
  uname -a
  echo "Memory Usage:"
  free -h
  ]]>
  </meta:shell>
  
  <!-- Conditional block - Processed by ConditionalRunner -->
  <meta:code language="python" name="check-threshold">
  <![CDATA[
  import json
  stats = json.loads('''<meta:reference target="data-processor" />''')
  print("true" if stats["filtered_percentage"] > 20 else "false")
  ]]>
  </meta:code>
  
  <meta:conditional if="check-threshold">
    <meta:code language="python" name="alert-processor">
    <![CDATA[
    import json
    stats = json.loads('''<meta:reference target="data-processor" />''')
    print(f"ALERT: High threshold rate detected: {stats['filtered_percentage']:.2f}%")
    ]]>
    </meta:code>
  </meta:conditional>
  
  <!-- Question block - Processed by QuestionRunner -->
  <meta:question name="data-insights" model="gpt-4" test_mode="true">
  Analyze this statistical data and provide key insights:
  <meta:reference target="data-processor" />
  </meta:question>
</meta:document>
```

When processed by the modular executor:
1. The `PythonRunner` executes the Python code blocks
2. The `ShellRunner` executes the shell commands
3. The `ConditionalRunner` checks if the condition is true and executes conditional content
4. The `QuestionRunner` processes the LLM prompt (or returns test mode response)

Each runner has specialized logic for its block type while sharing the common `ExecutorState`.

## Testing Deep Nested References

This example demonstrates comprehensive testing of deeply nested reference resolution:

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <!-- Base data structures -->
  <meta:data name="system-info" format="json">
  <![CDATA[
  {
    "name": "Production Server",
    "environment": "production",
    "version": "2.1.3",
    "resources": {
      "memory": "16GB",
      "cpu": "8 cores",
      "storage": "500GB"
    }
  }
  ]]>
  </meta:data>
  
  <meta:data name="database-config" format="json">
  <![CDATA[
  {
    "host": "db.example.com",
    "port": 5432,
    "database": "analytics",
    "connection_pool": 20,
    "timeout": 30
  }
  ]]>
  </meta:data>
  
  <meta:secret name="db-password">
  DB_PASSWORD_ENV
  </meta:secret>
  
  <!-- Level 1: Simple template references -->
  <meta:template name="system-banner">
  =============================================
  System: <meta:reference target="system-info.name" />
  Environment: <meta:reference target="system-info.environment" />
  Version: <meta:reference target="system-info.version" />
  =============================================
  </meta:template>
  
  <meta:template name="db-connection-string">
  postgresql://admin:<meta:reference target="db-password" />@<meta:reference target="database-config.host" />:<meta:reference target="database-config.port" />/<meta:reference target="database-config.database" />?timeout=<meta:reference target="database-config.timeout" />
  </meta:template>
  
  <!-- Level 2: References that reference templates -->
  <meta:variable name="diagnostics-header">
  <meta:reference target="system-banner" />
  
  DIAGNOSTICS REPORT
  Generated: ${new Date().toISOString()}
  </meta:variable>
  
  <meta:code language="python" name="connection-test">
  <![CDATA[
  import psycopg2
  
  # Connection string from template with nested refs
  connection_string = '''<meta:reference target="db-connection-string" />'''
  
  print(f"Testing connection to: {connection_string.split('@')[1].split('?')[0]}")
  # Actual connection code would go here in production
  print("Connection successful")
  ]]>
  </meta:code>
  
  <!-- Level 3: Deep references -->
  <meta:shell name="generate-report">
  <![CDATA[
  echo '<meta:reference target="diagnostics-header" />'
  echo ""
  echo "DATABASE CONNECTION STATUS:"
  echo '<meta:reference target="connection-test" />'
  echo ""
  echo "SYSTEM RESOURCES:"
  echo "Memory: <meta:reference target="system-info.resources.memory" />"
  echo "CPU: <meta:reference target="system-info.resources.cpu" />"
  echo "Storage: <meta:reference target="system-info.resources.storage" />"
  ]]>
  </meta:shell>
  
  <!-- Level 4: Ultra-deep reference -->
  <meta:code language="python" name="reference-resolution-test">
  <![CDATA[
  # This references a shell block that references:
  # 1. A variable containing a template with references
  # 2. A code block with template references
  # 3. Direct data references
  
  report = '''<meta:reference target="generate-report" />'''
  
  # Tests that all references were resolved correctly
  assert "System: Production Server" in report
  assert "Environment: production" in report
  assert "Version: 2.1.3" in report
  assert "Testing connection to: db.example.com:5432/analytics" in report
  assert "Connection successful" in report
  assert "Memory: 16GB" in report
  assert "CPU: 8 cores" in report
  assert "Storage: 500GB" in report
  
  print("✅ All nested references successfully resolved!")
  print("\nFinal combined output:\n")
  print(report)
  ]]>
  </meta:code>
</meta:document>
```

This example tests the following aspects of the reference resolution system:

1. **Multi-Level Resolution**: References within references, up to 4 levels deep
2. **Mixed Block Types**: References across different block types (data, variable, template, code, shell)
3. **JSON Path Access**: Dotted path navigation into JSON objects
4. **Secret Substitution**: Environment variable reference resolution
5. **Template Composition**: Building complex templates from nested references
6. **Self-Verification**: Code that validates its own reference resolution

The multi-pass resolution algorithm ensures that even the deepest references are fully resolved before the final execution.

## Complex Example: Cybersecurity Analysis

### XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="target-app" format="json" always_include="true">
  <![CDATA[
  {
    "url": "https://example-app.com",
    "tech_stack": ["Python", "Django", "PostgreSQL"],
    "authentication": true
  }
  ]]>
  </meta:data>
  
  <meta:api name="security-headers" method="GET" cache_result="true" retry="2" timeout="10" fallback="security-headers-fallback" debug="true">
  https://securityheaders
