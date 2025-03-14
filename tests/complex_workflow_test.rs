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
