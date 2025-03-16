# XML Format Examples

This document provides examples of Meta Programming Language documents in XML format.

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
  numbers = json.loads('''${numbers}''')
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
    data = json.loads('''${sales-data}''')
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
    <meta:question model="${model}" temperature="${temperature}">
    Analyze this dataset: ${dataset}
    </meta:question>
  </meta:template>
  
  <meta:template-invocation name="insights-invocation" template="data-insights">
    <meta:param name="dataset">${sales-data}</meta:param>
  </meta:template-invocation>
</meta:document>
```

## Complex Example: Cybersecurity Analysis

```

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
