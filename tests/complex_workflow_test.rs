#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document};
    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;

    /// Test a complete end-to-end workflow with multiple dependent blocks
    #[test]
    fn test_complex_workflow_with_dependencies() {
        let input = r#"[data name:target-app format:json always_include:true]
{
  "url": "https://example-app.com",
  "tech_stack": ["Python", "Django", "PostgreSQL"],
  "authentication": true
}
[/data]

[api name:security-headers method:GET cache_result:true retry:2 timeout:10 fallback:security-headers-fallback]
https://securityheaders.com/?url=${target-app.url}&format=json
[/api]

[data name:security-headers-fallback format:json]
{
  "headers": [],
  "grade": "unknown"
}
[/data]

[shell name:nmap-scan cache_result:true timeout:20 fallback:nmap-scan-fallback]
nmap -Pn -p 1-1000 ${target-app.url}
[/shell]

[data name:nmap-scan-fallback format:text]
Failed to scan ports. Using fallback data.
PORT   STATE  SERVICE
22/tcp open   ssh
80/tcp open   http
443/tcp open   https
[/data]

[code:python name:security-analysis depends:security-headers fallback:analysis-fallback]
import json

headers = json.loads('''${security-headers}''')
scan_results = '''${nmap-scan}'''

vulnerabilities = []
if headers.get("grade") != "A+":
    vulnerabilities.append("Insufficient security headers")

if "443/tcp" not in scan_results:
    vulnerabilities.append("HTTPS not detected")

print(f"Found {len(vulnerabilities)} potential issues")
for vuln in vulnerabilities:
    print(f"- {vuln}")
[/code:python]

[code:python name:analysis-fallback]
print("Security analysis could not be completed")
print("- Using fallback data")
print("- Recommend manual review")
[/code:python]

[question name:security-review depends:security-analysis]
Based on the security analysis, what are the key vulnerabilities that need addressing?
[/question]"#;

        let blocks = parse_document(input).unwrap();
        
        // Verify the number of blocks
        assert_eq!(blocks.len(), 7);
        
        // Check the dependency resolution in the security analysis block
        let analysis_block = blocks.iter().find(|b| b.name == Some("security-analysis".to_string())).unwrap();
        let dependencies = analysis_block.get_modifier("depends").unwrap();
        
        // Should contain both dependencies
        assert!(dependencies.contains("security-headers"));
        assert!(dependencies.contains("nmap-scan"));
        
        // Check that the analysis has a fallback defined
        assert_eq!(analysis_block.get_modifier("fallback"), Some(&"analysis-fallback".to_string()));
        
        // Check that the question block depends on the analysis
        let question_block = blocks.iter().find(|b| b.name == Some("security-review".to_string())).unwrap();
        assert_eq!(question_block.get_modifier("depends"), Some(&"security-analysis".to_string()));
    }
}
use std::collections::HashMap;
use std::time::Instant;

use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
use yet_another_llm_project_but_better::parser::Block;
use yet_another_llm_project_but_better::parser::parse_document;

#[test]
fn test_complex_workflow() {
    // Create a test document with multiple nested sections and blocks
    let test_document = r#"
[section:workflow name:data-processing-workflow]
# Complex Data Processing Workflow

This workflow demonstrates a multi-stage data processing pipeline with dependencies.

[code:python name:generate-data depends:none cache_result:true]
import json
import random

# Generate some random data
data = [random.randint(1, 100) for _ in range(10)]
print(json.dumps(data))
[/code:python]

[code:python name:process-data depends:generate-data]
import json
import statistics

# Process the data from the previous step
data = ${generate-data.results}
mean = statistics.mean(data)
median = statistics.median(data)
stdev = statistics.stdev(data) if len(data) > 1 else 0

result = {
    "mean": mean,
    "median": median,
    "stdev": stdev,
    "min": min(data),
    "max": max(data)
}

print(json.dumps(result))
[/code:python]

[section:visualization name:data-viz]
## Data Visualization

[code:python name:create-chart depends:process-data]
import json
import matplotlib.pyplot as plt
import io
import base64

# Get the processed data
stats = ${process-data.results}
raw_data = ${generate-data.results}

# Create a simple visualization
plt.figure(figsize=(10, 6))
plt.bar(range(len(raw_data)), raw_data)
plt.axhline(y=stats["mean"], color='r', linestyle='-', label=f'Mean: {stats["mean"]:.2f}')
plt.axhline(y=stats["median"], color='g', linestyle='--', label=f'Median: {stats["median"]:.2f}')
plt.legend()
plt.title('Data Distribution')

# Instead of saving to file, print confirmation
print("Chart created successfully")
[/code:python]
[/section:visualization]

[section:error-handling]
## Error Handling and Fallbacks

[code:python name:might-fail timeout:2]
import time
import random

# This might fail or timeout
if random.random() < 0.5:
    time.sleep(3)  # Should trigger timeout
    print("This should never be reached due to timeout")
else:
    print("Success: Operation completed without timeout")
[/code:python]

[data name:fallback-data]
{"status": "fallback", "message": "Using fallback data"}
[/data]

[code:python name:use-fallback-data depends:might-fail fallback:fallback-data]
import json

try:
    # Try to use the result from might-fail
    result = ${might-fail.results}
    print(f"Got result from previous step: {result}")
except:
    # This should use the fallback
    fallback = ${fallback-data}
    print(f"Using fallback: {fallback}")
[/code:python]
[/section:error-handling]

[section:conditional-logic]
## Conditional Logic

[variable name:threshold]
50
[/variable]

[code:python name:conditional-processing depends:generate-data]
import json

data = ${generate-data.results}
threshold = int(${threshold})

# Filter based on threshold
filtered = [x for x in data if x > threshold]
print(json.dumps({
    "original_count": len(data),
    "filtered_count": len(filtered),
    "filtered_data": filtered
}))
[/code:python]

[conditional if:${conditional-processing.results.filtered_count} > 0]
[code:python name:process-filtered depends:conditional-processing]
import json

result = ${conditional-processing.results}
filtered = result["filtered_data"]
average = sum(filtered) / len(filtered) if filtered else 0

print(f"Average of filtered data: {average}")
[/code:python]
[/conditional]
[/section:conditional-logic]

[/section:workflow]
"#;

    // Parse the document
    let blocks = parse_document(test_document).expect("Failed to parse document");
    
    // Create an executor
    let mut executor = MetaLanguageExecutor::new();
    
    // Register all blocks with the executor
    for block in &blocks {
        register_block_and_children(&mut executor, block);
    }
    
    // Execute the workflow
    let result = executor.execute_block("data-processing-workflow");
    assert!(result.is_ok(), "Workflow execution failed: {:?}", result.err());
    
    // Verify that generate-data was executed
    assert!(executor.outputs.contains_key("generate-data.results"), 
            "generate-data block was not executed");
    
    // Verify that process-data was executed and depends on generate-data
    assert!(executor.outputs.contains_key("process-data.results"), 
            "process-data block was not executed");
    
    // Parse the process-data results as JSON to verify structure
    let process_data_result = &executor.outputs["process-data.results"];
    let json_result: serde_json::Value = serde_json::from_str(process_data_result)
        .expect("Failed to parse process-data results as JSON");
    
    // Verify the JSON structure has the expected fields
    assert!(json_result.get("mean").is_some(), "Missing 'mean' in results");
    assert!(json_result.get("median").is_some(), "Missing 'median' in results");
    assert!(json_result.get("stdev").is_some(), "Missing 'stdev' in results");
    assert!(json_result.get("min").is_some(), "Missing 'min' in results");
    assert!(json_result.get("max").is_some(), "Missing 'max' in results");
    
    // Verify that create-chart was executed
    assert!(executor.outputs.contains_key("create-chart.results"), 
            "create-chart block was not executed");
    
    // Verify fallback mechanism
    let use_fallback_result = executor.execute_block("use-fallback-data");
    assert!(use_fallback_result.is_ok(), "Fallback execution failed");
    
    // Verify conditional logic
    let conditional_result = executor.execute_block("conditional-processing");
    assert!(conditional_result.is_ok(), "Conditional processing failed");
    
    // Check if process-filtered was executed (may or may not be, depending on random data)
    let filtered_result = executor.execute_block("process-filtered");
    println!("Filtered result: {:?}", filtered_result);
    
    // Verify that the cache works for generate-data
    let cached_time = executor.cache.get("generate-data")
        .map(|(_, time)| *time);
    assert!(cached_time.is_some(), "generate-data was not cached");
}

// Helper function to recursively register blocks and their children
fn register_block_and_children(executor: &mut MetaLanguageExecutor, block: &Block) {
    // Register this block
    if let Some(name) = &block.name {
        executor.blocks.insert(name.clone(), block.clone());
    }
    
    // Register all children recursively
    for child in &block.children {
        register_block_and_children(executor, child);
    }
}
