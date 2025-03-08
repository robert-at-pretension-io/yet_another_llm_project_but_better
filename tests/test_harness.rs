#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use yet_another_llm_project_but_better::*;

    #[test]
    fn test_parse_document_success() {
        let document = "[question name:sample]\nWhat is the answer?\n[/question]";
        let doc = parse_document(document).expect("Failed to parse document");
        assert!(doc.blocks.contains_key("sample"), "Document should contain a block named 'sample'");
    }

    #[test]
    fn test_resolve_dependencies() {
        let document = "[question name:sample depends:other]\nWhat?\n[/question][question name:other]\nOther question?\n[/question]";
        let doc = parse_document(document).expect("Failed to parse document");
        let deps: HashSet<String> = doc.dependencies.get("sample").cloned().unwrap_or_default();
        assert!(deps.contains("other"), "Dependencies should contain 'other'");
    }

    #[test]
    fn test_process_questions_no_error() {
        // Create a simple document with a question block.
        let document = "[question name:test_question model:default debug:true]\nCompute X\n[/question]";
        let mut doc = parse_document(document).expect("Failed to parse document");
        // Process questions which should generate a response block.
        process_questions(&mut doc).expect("Processing questions failed");
        // Check that a response block was added.
        let response_key = "test_question-response";
        assert!(doc.blocks.contains_key(response_key), "Response block should be added");
    }
    #[test]
    fn test_large_file_parsing() {
        let large_file = r#"[data name:project-info format:json always_include:true]
{
  "name": "Test Project",
  "version": "0.1.0",
  "description": "Testing the meta language parser"
}
[/data]

[comment]
This is a comment block that should be parsed but not executed.
The parser should handle multi-line content properly.
[/comment]

[variable name:max-retries]
3
[/variable]

[variable name:base-url]
https://api.example.com/v1
[/variable]

[code:python name:basic-python fallback:python-fallback]
# A simple Python code block
def hello_world():
    print("Hello, World!")
    
hello_world()
[/code:python]

[code:python name:python-fallback]
print("This is a fallback block")
[/code:python]

[code:javascript name:javascript-example fallback:js-fallback]
// JavaScript sample
function calculateSum(arr) {
  return arr.reduce((a, b) => a + b, 0);
}

console.log(calculateSum([1, 2, 3, 4, 5]));
[/code:javascript]

[code:javascript name:js-fallback]
console.log("JavaScript fallback executed");
[/code:javascript]

[shell name:list-files fallback:shell-fallback]
ls -la
[/shell]

[shell name:shell-fallback]
echo "Shell command failed"
[/shell]

[api name:get-users method:GET timeout:5 cache_result:true fallback:api-fallback]
${base-url}/users
[/api]

[data name:api-fallback format:json]
{
  "error": "API fallback triggered",
  "users": []
}
[/data]

[question name:simple-question model:gpt-4]
What is the capital of France?
[/question]

[question name:data-question model:gpt-4 depends:project-info]
Analyze the following project data and suggest next steps:
${project-info}
[/question]

[question name:complex-question model:gpt-4 temperature:0.7 depends:get-users]
Based on the user data:
${get-users}

What demographics do you observe?
[/question]

[template name:api-request model:gpt-4 temperature:0.3]
[question model:${model} temperature:${temperature}]
Analyze this API response:
${${name}}

${additional_instructions}
[/question]
[/template]

[@api-request 
  name:"weather-data" 
  method:"GET" 
  fallback:"weather-fallback" 
  url:"https://api.weather.example/current" 
  model:"gpt-4"
  temperature:"0.2"
  additional_instructions:"Focus on temperature trends."
]
[/@api-request]

[data name:weather-fallback format:json]
{
  "error": "Weather API fallback",
  "temp": 20,
  "conditions": "unknown"
}
[/data]

[section:analysis name:performance-analysis]
  [code:python name:generate-data fallback:generate-data-fallback]
  import random
  import json
  
  data = [{'day': i, 'value': random.randint(1, 100)} for i in range(30)]
  print(json.dumps(data))
  [/code:python]
  
  [code:python name:generate-data-fallback]
  print('[{"day": 1, "value": 50}]')
  [/code:python]
  
  [question name:performance-insight model:gpt-4 depends:generate-data]
  Analyze this performance data and provide insights:
  ${generate-data}
  [/question]
[/section]

[visualization]
  [question model:gpt-4 debug:true]
  Summarize the project status based on all available data.
  Consider performance metrics, user data, and project information.
  [/question]

  [preview]
  (Preview will be generated here by the system)
  [/preview]
[/visualization]

[error type:execution_failure]
An error occurred while processing the block 'missing-block'.
Execution stopped. Please ensure the block exists and all dependencies are satisfied.
[/error]

[secret name:api-key]
API_KEY_ENV_VAR
[/secret]

[code:python name:empty-block fallback:empty-fallback]
[/code:python]

[code:python name:empty-fallback]
print("Empty block fallback executed")
[/code:python]

[code:python name:special-chars fallback:special-chars-fallback]
# Testing special characters
print("Testing: !@#$%^&*()_+-=[]{}|;':\",./<>?")
print("""
Multi-line
string with "quotes"
and [brackets]
""")
[/code:python]

[data name:special-chars-fallback]
Fallback content for special characters test
[/data]

[question name:already-answered-question model:gpt-4]
What is 2+2?
[/question]

[response name:already-answered-question-response]
The answer to 2+2 is 4.

This is a simple arithmetic addition. The number 2 added to itself gives 4.
[/response]

[question name:kitchen-sink-test
  model:gpt-4
  temperature:0.7
  max_tokens:2048
  top_p:0.9
  frequency_penalty:0.5
  presence_penalty:0.5
  timeout:30
  retry:2
  cache_result:true
  always_include:false
  priority:8
  order:0.5
  weight:0.8
  summarize:brief
  debug:true
]
This block tests the parser's ability to handle many modifiers at once.
[/question]

[data name:xml-content format:xml]
<?xml version="1.0" encoding="UTF-8"?>
<root>
  <item id="1">
    <name>First Item</name>
    <value>100</value>
  </item>
  <item id="2">
    <name>Second Item</name>
    <value>200</value>
  </item>
</root>
[/data]

[data name:html-content format:html]
<!DOCTYPE html>
<html>
<head>
  <title>Test Page</title>
</head>
<body>
  <h1>Hello World</h1>
  <p>This is a test</p>
  <script>
    // This should be properly handled
    const x = 10;
    if (x > 5) {
      console.log("x is greater than 5");
    }
  </script>
</body>
</html>
[/data]
    }
}
