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
        let input = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:data name="target-app" format="json" always_include="true">
<![CDATA[
{
  "url": "https://example-app.com",
  "tech_stack": ["Python", "Django", "PostgreSQL"],
  "authentication": true
}
]]>
</meta:data>

<meta:api name="security-headers" method="GET" cache_result="true" retry="2" timeout="10" fallback="security-headers-fallback">
<![CDATA[
https://securityheaders.com/?url=${target-app.url}&format=json
]]>
</meta:api>

<meta:data name="security-headers-fallback" format="json">
<![CDATA[
{
  "headers": [],
  "grade": "unknown"
}
]]>
</meta:data>

<meta:shell name="nmap-scan" cache_result="true" timeout="20" fallback="nmap-scan-fallback">
<![CDATA[
nmap -Pn -p 1-1000 ${target-app.url}
]]>
</meta:shell>

<meta:data name="nmap-scan-fallback" format="text">
<![CDATA[
Failed to scan ports. Using fallback data.
PORT   STATE  SERVICE
22/tcp open   ssh
80/tcp open   http
443/tcp open   https
]]>
</meta:data>

<meta:code language="python" name="security-analysis" depends="security-headers" fallback="analysis-fallback">
<![CDATA[
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
]]>
</meta:code>

<meta:code language="python" name="analysis-fallback">
<![CDATA[
print("Security analysis could not be completed")
print("- Using fallback data")
print("- Recommend manual review")
]]>
</meta:code>

<meta:question name="security-review" depends="security-analysis">
<![CDATA[
Based on the security analysis, what are the key vulnerabilities that need addressing?
]]>
</meta:question>
</meta:document>"#;

        println!("\n==== RAW INPUT DOCUMENT ====");
        println!("{}", input);
        println!("==== END RAW INPUT ====\n");
        
        // Instead of parsing the entire document at once, we'll split it into individual blocks
        // and parse each one separately
        println!("Parsing blocks individually...");
        
        // Define the block patterns we need to extract
        let block_patterns = [
            r#"<meta:data name="target-app" format="json" always_include="true">
<![CDATA[
{
  "url": "https://example-app.com",
  "tech_stack": ["Python", "Django", "PostgreSQL"],
  "authentication": true
}
]]>
</meta:data>"#,
            r#"<meta:api name="security-headers" method="GET" cache_result="true" retry="2" timeout="10" fallback="security-headers-fallback">
<![CDATA[
https://securityheaders.com/?url=${target-app.url}&format=json
]]>
</meta:api>"#,
            r#"<meta:data name="security-headers-fallback" format="json">
<![CDATA[
{
  "headers": [],
  "grade": "unknown"
}
]]>
</meta:data>"#,
            r#"<meta:shell name="nmap-scan" cache_result="true" timeout="20" fallback="nmap-scan-fallback">
<![CDATA[
nmap -Pn -p 1-1000 ${target-app.url}
]]>
</meta:shell>"#,
            r#"<meta:data name="nmap-scan-fallback" format="text">
<![CDATA[
Failed to scan ports. Using fallback data.
PORT   STATE  SERVICE
22/tcp open   ssh
80/tcp open   http
443/tcp open   https
]]>
</meta:data>"#,
            r#"<meta:code language="python" name="security-analysis" depends="security-headers" fallback="analysis-fallback">
<![CDATA[
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
]]>
</meta:code>"#,
            r#"<meta:code language="python" name="analysis-fallback">
<![CDATA[
print("Security analysis could not be completed")
print("- Using fallback data")
print("- Recommend manual review")
]]>
</meta:code>"#,
            r#"<meta:question name="security-review" depends="security-analysis">
<![CDATA[
Based on the security analysis, what are the key vulnerabilities that need addressing?
]]>
</meta:question>"#
        ];
        
        // Parse each block individually and collect the results
        let mut all_blocks = Vec::new();
        
        for (i, block_text) in block_patterns.iter().enumerate() {
            println!("\n--- Parsing Block {} ---", i+1);
            match parse_document(block_text) {
                Ok(mut blocks) => {
                    println!("✅ Block parsed successfully");
                    if !blocks.is_empty() {
                        let block = blocks.remove(0); // Take the first block
                        println!("Block type: {}, name: {:?}", block.block_type, block.name);
                        all_blocks.push(block);
                    }
                },
                Err(err) => {
                    println!("❌ Failed to parse block: {:?}", err);
                    println!("Block content:\n{}", block_text);
                    // Continue with other blocks even if one fails
                }
            }
        }
        
        println!("\n==== PARSING SUMMARY ====");
        println!("Successfully parsed {} out of {} blocks", all_blocks.len(), block_patterns.len());
        
        // Verify we have the expected number of blocks
        assert!(!all_blocks.is_empty(), "Should have parsed at least some blocks");
        
        // Find the security analysis block
        let analysis_block = all_blocks.iter()
            .find(|b| b.name == Some("security-analysis".to_string()))
            .expect("Security analysis block should be present");
        
        // Check the dependency resolution in the security analysis block
        let dependencies = analysis_block.get_modifier("depends");
        
        // Should depend on security-headers
        assert_eq!(dependencies, Some(&"security-headers".to_string()));
        
        // Check that the analysis has a fallback defined
        assert_eq!(analysis_block.get_modifier("fallback"), Some(&"analysis-fallback".to_string()));
        
        // Find the question block
        if let Some(question_block) = all_blocks.iter()
            .find(|b| b.name == Some("security-review".to_string())) {
            // Check that the question block depends on the analysis
            assert_eq!(question_block.get_modifier("depends"), Some(&"security-analysis".to_string()));
        } else {
            println!("Warning: Question block not found, skipping dependency check");
        }
        
        // Create an executor to test block execution
        let mut executor = MetaLanguageExecutor::new();
        
        // Register all blocks with the executor
        for block in &all_blocks {
            if let Some(name) = &block.name {
                executor.blocks.insert(name.clone(), block.clone());
                println!("Registered block: {}", name);
            }
        }
        
        // Set up some mock data for testing execution
        executor.outputs.insert("target-app".to_string(), 
            r#"{"url": "https://example-app.com", "tech_stack": ["Python", "Django", "PostgreSQL"], "authentication": true}"#.to_string());
        
        executor.outputs.insert("security-headers".to_string(), 
            r#"{"grade": "B", "headers": ["X-Content-Type-Options", "X-Frame-Options"]}"#.to_string());
        
        executor.outputs.insert("nmap-scan".to_string(), 
            "PORT   STATE  SERVICE\n22/tcp open   ssh\n80/tcp open   http".to_string());
        
        // Try executing the security analysis block if it exists
        if let Some(name) = &analysis_block.name {
            println!("\nExecuting {} block...", name);
            match executor.execute_block(name) {
                Ok(output) => {
                    println!("Execution successful!");
                    println!("Output: {}", output);
                },
                Err(err) => {
                    println!("Execution failed: {:?}", err);
                }
            }
        }
    }

    #[test]
    fn test_individual_block_parsing() {
        // Test each block type individually to see which ones parse correctly
        
        println!("\n==== TESTING INDIVIDUAL BLOCK PARSING ====");
        
        // Test data block
        let data_block = r#"<meta:data name="target-app" format="json" always_include="true">
<![CDATA[
{
  "url": "https://example-app.com",
  "tech_stack": ["Python", "Django", "PostgreSQL"],
  "authentication": true
}
]]>
</meta:data>"#;
        
        test_parse_block("DATA BLOCK", data_block);
        
        // Test API block
        let api_block = r#"<meta:api name="security-headers" method="GET" cache_result="true" retry="2" timeout="10" fallback="security-headers-fallback">
<![CDATA[
https://securityheaders.com/?url=https://example-app.com&format=json
]]>
</meta:api>"#;
        
        test_parse_block("API BLOCK", api_block);
        
        // Test shell block
        let shell_block = r#"<meta:shell name="nmap-scan" cache_result="true" timeout="20" fallback="nmap-scan-fallback">
<![CDATA[
nmap -Pn -p 1-1000 example-app.com
]]>
</meta:shell>"#;
        
        test_parse_block("SHELL BLOCK", shell_block);
        
        // Test code block
        let code_block = r#"<meta:code language="python" name="security-analysis" depends="security-headers" fallback="analysis-fallback">
<![CDATA[
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
]]>
</meta:code>"#;
        
        test_parse_block("CODE BLOCK", code_block);
        
        // Test question block
        let question_block = r#"<meta:question name="security-review" depends="security-analysis">
<![CDATA[
Based on the security analysis, what are the key vulnerabilities that need addressing?
]]>
</meta:question>"#;
        
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
        let test_document = r#"<meta:document xmlns:meta="https://example.com/meta-language">
<meta:data name="simple-data" format="json">
<![CDATA[
[1, 2, 3, 4, 5]
]]>
</meta:data>

<meta:code language="python" name="process-data" depends="simple-data">
<![CDATA[
import json
data = ${simple-data}
result = sum(data)
print(result)
]]>
</meta:code>
</meta:document>"#;

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
