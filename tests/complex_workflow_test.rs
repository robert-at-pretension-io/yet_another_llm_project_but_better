#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Instant;

    use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
    use yet_another_llm_project_but_better::parser::parse_document;
    use yet_another_llm_project_but_better::parser::Block;

    /// Test a complete end-to-end workflow with multiple dependent blocks
    #[test]
    fn test_complex_workflow_with_dependencies() {
        // Test parsing a complex document with dependencies
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
        
        // Verify the number of blocks (top-level blocks)
        assert_eq!(blocks.len(), 8);
        
        // Check the dependency resolution in the security analysis block
        let analysis_block = blocks.iter().find(|b| b.name == Some("security-analysis".to_string())).unwrap();
        let dependencies = analysis_block.get_modifier("depends");
        
        // Should depend on security-headers
        assert_eq!(dependencies, Some(&"security-headers".to_string()));
        
        // Check that the analysis has a fallback defined
        assert_eq!(analysis_block.get_modifier("fallback"), Some(&"analysis-fallback".to_string()));
        
        // Check that the question block depends on the analysis
        let question_block = blocks.iter().find(|b| b.name == Some("security-review".to_string())).unwrap();
        assert_eq!(question_block.get_modifier("depends"), Some(&"security-analysis".to_string()));
    }

    #[test]
    fn test_simple_workflow() {
        // Create a test document with just a few blocks for testing
        let test_document = r#"
[data name:simple-data format:json]
[1, 2, 3, 4, 5]
[/data]

[code:python name:process-data depends:simple-data]
import json
data = ${simple-data}
result = sum(data)
print(result)
[/code:python]
"#;

        // Parse the document
        let blocks = parse_document(test_document).expect("Failed to parse document");
        
        // Create an executor
        let mut executor = MetaLanguageExecutor::new();
        
        // Register all blocks with the executor
        for block in &blocks {
            register_block_and_children(&mut executor, block);
        }
        
        // Manually set up the data block result
        executor.outputs.insert("simple-data".to_string(), "[1, 2, 3, 4, 5]".to_string());
        
        // Execute the process-data block
        println!("Executing process-data block...");        let result = executor.execute_block("process-data");
        assert!(result.is_ok(), "Process data execution failed: {:?}", result.err());
        
        if let Ok(output) = result {
            assert_eq!(output.trim(), "15", "Sum of [1,2,3,4,5] should be 15");
            println!("Process data output: {}", output);
        }
        
        // Verify that the output was stored correctly
        assert!(executor.outputs.contains_key("process-data.results"), 
                "process-data block results should be stored");
    }

    // Helper function to recursively register blocks and their children
    fn register_block_and_children(executor: &mut MetaLanguageExecutor, block: &Block) {
        // Register this block
        if let Some(name) = &block.name {
            executor.blocks.insert(name.clone(), block.clone());
            println!("Registered block: {}", name);
        }
        
        // Register all children recursively
        for child in &block.children {
            register_block_and_children(executor, child);
        }
    }
}
