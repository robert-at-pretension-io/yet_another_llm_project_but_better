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

        println!("\n==== RAW INPUT DOCUMENT ====");
        println!("{}", input);
        println!("==== END RAW INPUT ====\n");
        
        // Try to parse the document and capture the result
        let parse_result = parse_document(input);
        
        // Check if parsing succeeded or failed
        match &parse_result {
            Ok(blocks) => {
                println!("✅ Document parsing succeeded!");
                println!("Number of blocks found: {}", blocks.len());
                
                // Print detailed information about each block
                for (i, block) in blocks.iter().enumerate() {
                    println!("\n🔍 Block {}: type={}, name={:?}", 
                        i, block.block_type, block.name);
                    println!("  Modifiers: {:?}", block.modifiers);
                    println!("  Content length: {} chars", block.content.len());
                    println!("  Content preview: {}", 
                        if block.content.len() > 50 { 
                            format!("{}...", &block.content[..50]) 
                        } else { 
                            block.content.clone() 
                        });
                    println!("  Children count: {}", block.children.len());
                }
            },
            Err(err) => {
                println!("❌ Document parsing failed: {:?}", err);
            }
        }
        
        // Unwrap the result to continue with the test
        let blocks = parse_result.unwrap();
        
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
    fn test_individual_block_parsing() {
        // Test each block type individually to see which ones parse correctly
        
        println!("\n==== TESTING INDIVIDUAL BLOCK PARSING ====");
        
        // Test data block
        let data_block = r#"[data name:target-app format:json always_include:true]
{
  "url": "https://example-app.com",
  "tech_stack": ["Python", "Django", "PostgreSQL"],
  "authentication": true
}
[/data]"#;
        
        test_parse_block("DATA BLOCK", data_block);
        
        // Test API block
        let api_block = r#"[api name:security-headers method:GET cache_result:true retry:2 timeout:10 fallback:security-headers-fallback]
https://securityheaders.com/?url=https://example-app.com&format=json
[/api]"#;
        
        test_parse_block("API BLOCK", api_block);
        
        // Test shell block
        let shell_block = r#"[shell name:nmap-scan cache_result:true timeout:20 fallback:nmap-scan-fallback]
nmap -Pn -p 1-1000 example-app.com
[/shell]"#;
        
        test_parse_block("SHELL BLOCK", shell_block);
        
        // Test code block
        let code_block = r#"[code:python name:security-analysis depends:security-headers fallback:analysis-fallback]
import json

headers = json.loads('''{"grade": "B"}''')
scan_results = '''PORT   STATE  SERVICE
22/tcp open   ssh
80/tcp open   http'''

vulnerabilities = []
if headers.get("grade") != "A+":
    vulnerabilities.append("Insufficient security headers")

if "443/tcp" not in scan_results:
    vulnerabilities.append("HTTPS not detected")

print(f"Found {len(vulnerabilities)} potential issues")
for vuln in vulnerabilities:
    print(f"- {vuln}")
[/code:python]"#;
        
        test_parse_block("CODE BLOCK", code_block);
        
        // Test question block
        let question_block = r#"[question name:security-review depends:security-analysis]
Based on the security analysis, what are the key vulnerabilities that need addressing?
[/question]"#;
        
        test_parse_block("QUESTION BLOCK", question_block);
    }
    
    // Helper function to test parsing a single block
    fn test_parse_block(block_type: &str, block_content: &str) {
        println!("\n--- Testing {} ---", block_type);
        println!("Content:\n{}", block_content);
        
        match parse_document(block_content) {
            Ok(blocks) => {
                println!("✅ Parsing succeeded! Found {} blocks", blocks.len());
                for (i, block) in blocks.iter().enumerate() {
                    println!("  Block {}: type={}, name={:?}", 
                        i, block.block_type, block.name);
                    println!("  Modifiers: {:?}", block.modifiers);
                }
            },
            Err(err) => {
                println!("❌ Parsing failed: {:?}", err);
            }
        }
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
        
        // Debug: Print the number of blocks found in simple workflow
        println!("\n==== SIMPLE WORKFLOW PARSING ====");
        println!("Number of blocks found: {}", blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            println!("Block {}: type={}, name={:?}", 
                i, block.block_type, block.name);
            println!("  Modifiers: {:?}", block.modifiers);
            println!("  Content preview: {}", 
                if block.content.len() > 30 { 
                    format!("{}...", &block.content[..30]) 
                } else { 
                    block.content.clone() 
                });
        }
        
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
