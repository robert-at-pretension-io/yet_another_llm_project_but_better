# 📚 Comprehensive User Tutorial for the Meta Programming Language

Welcome to your journey from complete beginner to expert user in the innovative meta-programming language—designed to combine AI interactions, executable code, and structured data seamlessly in a dynamic document.

## 🧭 Getting Started

### Understanding Blocks
Blocks are the basic building units:
- **Type**: Defines functionality (`code`, `question`, `data`, etc.)
- **Name** *(optional)*: Reference blocks easily
- **Modifiers** *(optional)*: Control execution and behavior

**Simple Example:**
```markdown
[code:python name:greet]
print("Hello World")
[/code:python]
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
```

### Shell Commands
Execute system-level operations:
```markdown
[shell name:list-directory]
ls -l
[/shell]
```

### API Interactions
Integrate with external APIs:
```markdown
[api name:get-info method:GET cache_result:true fallback:get-info-fallback]
https://api.example.com/info
[/api]

[data name:get-info-fallback format:json]
{"status": "unavailable"}
[/data]
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
```

The daemon auto-inserts these if omitted.

## ⚠️ Error Handling
Clearly handle and report errors:
```markdown
[error type:namespace_conflict]
Multiple blocks named "user-details" found. Execution stopped until resolved.
[/error]
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

## 🎓 Congratulations!
You now have everything you need to become an expert in our powerful meta-programming language. Dive in and start creating your dynamic, AI-enhanced documents today!

