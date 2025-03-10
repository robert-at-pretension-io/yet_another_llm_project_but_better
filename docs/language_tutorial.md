# 📚 Comprehensive User Tutorial for the Meta Programming Language

Welcome to your journey from complete beginner to expert user in the innovative meta-programming language—designed to combine AI interactions, executable code, and structured data seamlessly in a dynamic document.

## 🧭 Getting Started

### Understanding Blocks
Blocks are the basic building units:
- **Type**: Defines functionality (`code`, `question`, `data`, etc.)
- **Name**: Reference blocks easily. **Must be unique across the document.**
- **Modifiers** *(optional)*: Control execution and behavior

**Simple Example:**
```markdown
[code:python name:greet]
print("Hello World")
[/code:python]

[results for:greet format:plain]
Hello World
[/results]
```

## 🌳 Structuring Your Document

### Sections
Sections organize blocks hierarchically:
```markdown
[section:analysis name:sales-report]
  [data name:sales-data format:json]
  {"sales": 1000}
  [/data]
[/section]
```

## 💡 Interacting with AI

### Question-Response Pattern
Pose questions and get automated responses:
```markdown
[question model:gpt-4]
Explain recursion clearly.
[/question]

[response]
(Automatic AI response here)
[/response]
```

## 🛠️ Execution Blocks

### Code Execution
Run code directly within your document:
```markdown
[code:python name:calculate-sum]
print(sum([1, 2, 3, 4, 5]))
[/code:python]

[results for:calculate-sum format:plain]
15
[/results]
```

### Shell Commands
Execute system-level operations:
```markdown
[shell name:list-directory]
ls -l
[/shell]

[results for:list-directory format:plain]
total 12
drwxr-xr-x 2 user user 4096 Jan 15 10:30 docs
drwxr-xr-x 4 user user 4096 Jan 15 10:25 src
drwxr-xr-x 3 user user 4096 Jan 15 10:28 tests
[/results]
```

### API Interactions
Integrate with external APIs:
```markdown
[api name:get-info method:GET cache_result:true fallback:get-info-fallback]
https://api.example.com/info
[/api]

[results for:get-info format:json]
{
  "status": "success",
  "data": {
    "name": "Example API",
    "version": "1.0"
  }
}
[/results]

[data name:get-info-fallback format:json]
{"status": "unavailable"}
[/data]
```

## 📋 Results Blocks

### Automatic Results
Results are automatically generated after execution:
```markdown
[code:python name:analyze-data]
data = [10, 20, 30, 40, 50]
print(f"Average: {sum(data)/len(data)}")
print(f"Max: {max(data)}")
[/code:python]

[results for:analyze-data]
Average: 30.0
Max: 50
[/results]
```

### Customizing Results Display
Control how results appear:
```markdown
[code:python name:generate-table display:block format:markdown max_lines:10]
import pandas as pd

df = pd.DataFrame({'Name': ['Alice', 'Bob', 'Charlie'], 
                  'Score': [85, 92, 78]})
print(df.to_markdown())
[/code:python]

[results for:generate-table format:markdown]
|    | Name    |   Score |
|---:|:--------|--------:|
|  0 | Alice   |      85 |
|  1 | Bob     |      92 |
|  2 | Charlie |      78 |
[/results]
```

### Referencing Results
Use results in other blocks:
```markdown
[code:python name:get-numbers]
numbers = [1, 2, 3, 4, 5]
print(numbers)
[/code:python]

[results for:get-numbers]
[1, 2, 3, 4, 5]
[/results]

[code:python name:process-numbers]
prev_numbers = ${get-numbers.results}
total = sum(eval(prev_numbers))
print(f"The sum is: {total}")
[/code:python]

[results for:process-numbers]
The sum is: 15
[/results]
```

## 📦 Data Management

### Data Blocks
Store and reuse data:
```markdown
[data name:user-details format:json]
{"name": "Alex", "role": "admin"}
[/data]
```

### Variables
Reusable values for convenience:
```markdown
[variable name:max-users]
100
[/variable]
```

### Secrets
Securely manage sensitive data via environment variables:
```markdown
[secret name:api-key]
API_KEY_ENV_VAR
[/secret]
```

## 🎨 Templates
Efficiently reuse patterns with templates:

### Defining Templates
```markdown
[template name:data-insights model:gpt-4 temperature:0.3]
[question model:${model} temperature:${temperature}]
Analyze this dataset: ${dataset}
[/question]
[/template]
```

### Using Templates
```markdown
[@data-insights dataset:"${sales-data}"]
[/@data-insights]
```

## 🐞 Debugging and Troubleshooting

Enable detailed debugging:
```markdown
[debug enabled:true verbosity:high]
[/debug]
```

### Visualization and Preview
Preview context built for questions without triggering AI:
```markdown
[visualization]
  [question debug:true]
  Summarize key findings.
  [/question]

  [preview]
  (Preview generated here)
  [/preview]
[/visualization]
```

## 🔗 Dependencies and Execution Flow

### Explicit Dependencies
Declare dependencies explicitly to manage execution order:
```markdown
[question depends:calculate-sum]
Interpret the sum calculation result.
[/question]
```

## 🚩 Mandatory Fallbacks
All executable blocks must have fallbacks defined:
```markdown
[code:python name:data-loader fallback:data-loader-fallback]
load_data_from_source()
[/code:python]

[code:python name:data-loader-fallback]
print("Default data loaded")
[/code:python]

[results for:data-loader-fallback]
Default data loaded
[/results]
```

The daemon auto-inserts these if omitted.

## ⚠️ Error Handling
Clearly handle and report errors:
```markdown
[error type:namespace_conflict]
Multiple blocks named "user-details" found. Execution stopped until resolved.
[/error]
```

### Error Results
When execution fails, error results are shown:
```markdown
[code:python name:will-fail]
print(undefined_variable)
[/code:python]

[error_results for:will-fail]
NameError: name 'undefined_variable' is not defined
[/error_results]
```

## 🚦 Workflow Management

### Context and Token Management
- Use `priority`, `always_include`, and `order` modifiers.
- Apply summarization (`summarize:brief|semantic`) to stay within token limits.

## 🔄 Version Control and State Management
- Document changes auto-commit to Git.
- View or rollback to previous document states easily.

## 🚀 Tips to Master the Language
- Use descriptive naming consistently.
- Preview often to ensure accurate context.
- Enable debugging selectively when troubleshooting.
- Manage tokens and prioritize content effectively.
- Review execution results to verify expected behavior.
- Use appropriate formats for different result types.

## 🎓 Congratulations!
You now have everything you need to become an expert in our powerful meta-programming language. Dive in and start creating your dynamic, AI-enhanced documents today!