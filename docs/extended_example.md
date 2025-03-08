## Extended Multi-Block Cybersecurity Vulnerability Assessment

This complex example demonstrates a thorough cybersecurity vulnerability assessment workflow, leveraging multiple block dependencies, conditional execution, templating, debugging, error handling, and mandatory fallbacks.

---

### Step 1: Define Target Web Application
```markdown
[data name:target-app format:json always_include:true]
{
  "url": "https://example-app.com",
  "tech_stack": ["Python", "Django", "PostgreSQL"],
  "authentication": true
}
[/data]
```

### Step 2: Fetch Security Headers (External API Call)
```markdown
[api name:security-headers method:GET cache_result:true retry:2 timeout:10 fallback:security-headers-fallback debug:true]
https://securityheaders.com/?url=${target-app.url}&format=json
[/api]

[data name:security-headers-fallback format:json]
{
  "headers": [],
  "grade": "unknown"
}
[/data]
```

---

### Step 2: Scan Web Application for Open Ports
```markdown
[shell name:nmap-scan cache_result:true timeout:20 debug:true fallback:nmap-scan-fallback]
nmap -Pn -p 1-1000 ${target-app.url}
[/shell]

[error name:nmap-scan-fallback]
Nmap failed or timed out. Network scan unavailable.
[/error]
```

---

### Step 2: Analyze Codebase Vulnerabilities (Static Analysis)
```markdown
[code:python name:code-analysis cache_result:true depends:target-app debug:true fallback:code-analysis-fallback]
import subprocess

result = subprocess.run(
    ['bandit', '-r', '/path/to/codebase'],
    capture_output=True
)
print(result.stdout.decode())
[/code:python]

[code:python name:code-analysis-fallback]
print("No vulnerabilities found (fallback).");
[/code:python]
```

---

### Step 3: AI-based Security Recommendations Template
```markdown
[template name:security-recommendations model:gpt-4 temperature:0.2 max_tokens:700]
[question model:${model} temperature:${temperature}]
You are a cybersecurity analyst. Given:
- Security headers: ${security-headers}
- Network scan output: ${network-scan}
- Static analysis results: ${static-analysis}

Provide comprehensive recommendations prioritized by severity, formatted clearly.
[/question]
[/template]
```

---

### Step 4: Conditional Visualization Preview
This will generate a preview first, without directly calling the LLM:

```markdown
[visualization name:security-assessment-preview]
  [@security-recommendations
    static-analysis:"${code-analysis}"
    security-headers:"${security-headers}"
    network-scan:"${nmap-scan}"
  ]
[/visualization]

[preview]
<!-- Daemon auto-generates a complete prompt preview here -->
[/preview]
```

---

### Step 5: Execute Security Recommendations
After verifying the preview, execute:

```markdown
[@security-recommendations
  static-analysis:"${code-analysis}"
  security-headers:"${security-headers}"
  network-scan:"${nmap-scan}"
]
[/@security-recommendations]
```

---

### Error and Conflict Handling
If a namespace conflict occurs, the daemon explicitly halts execution:
```markdown
[error type:namespace_conflict]
Multiple blocks named "security-headers" detected. Resolve naming conflict.
[/error]
```

---

### Debugging Enabled
Throughout, `debug:true` flags enable clear visibility into each step, providing execution logs and dependencies.

---

## Workflow Dependency Graph:
```
target-app
├─ security-headers (API)
│  └─ fallback → security-headers-fallback
├─ nmap-scan (Shell)
│  └─ fallback → nmap-scan-fallback
└─ code-analysis (Code)
   └─ fallback → code-analysis-fallback

security-recommendations (Template)
├─ security-headers
├─ nmap-scan
└─ code-analysis
```

---

### Result
A robust, self-contained cybersecurity vulnerability assessment workflow with mandatory fallbacks, detailed debugging, and clear error handling for easy troubleshooting and high reliability.

