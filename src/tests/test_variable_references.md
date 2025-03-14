# Variable References Test

This document tests the variable reference functionality in the meta-language.

[code:python name:generate_data]
import json

data = {
    "name": "Test User",
    "age": 30,
    "skills": ["Rust", "Python", "JavaScript"]
}

print(json.dumps(data, indent=2))
[/code:python]

[code:python name:process_data depends:generate_data]
import json

# Parse the data from the previous block
data = json.loads("""${generate_data}""")

# Process the data
data["age"] += 1
data["skills"].append("Meta-Language")

print(json.dumps(data, indent=2))
[/code:python]

[code:python name:final_output depends:process_data]
import json

# Parse the data from the previous block
data = json.loads("""${process_data}""")

# Create a summary
summary = f"Name: {data['name']}, Age: {data['age']}, Skills: {len(data['skills'])}"
print(summary)
[/code:python]

## Expected Results:
- The `generate_data` block should produce a JSON object
- The `process_data` block should increment the age and add a skill
- The `final_output` block should create a summary string
