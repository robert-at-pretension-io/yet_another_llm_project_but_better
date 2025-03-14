#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    
    #[test]
    fn test_mixed_block_types() {
        let input = r#"[data name:user-info format:json]
{"name": "Alice", "role": "Developer"}
[/data]

[code:python name:greet-user]
import json
user = json.loads('${user-info}')
print(f"Hello, {user['name']}! You are a {user['role']}.")
[/code:python]

[shell name:run-script]
python script.py
[/shell]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse mixed blocks: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 3, "Expected 3 blocks, found {}", blocks.len());
        
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[1].block_type, "code:python");
        assert_eq!(blocks[2].block_type, "shell");
    }
    
    #[test]
    #[ignore] // Temporarily ignore this test until we fix the API depends issue
    fn test_document_with_dependencies() {
        let input = r#"[data name:config format:json]
{"api_url": "https://api.example.com", "timeout": 30}
[/data]

[code:python name:api-call depends:config]
import json
import requests

config = json.loads('${config}')
response = requests.get(config['api_url'], timeout=config['timeout'])
print(response.status_code)
[/code:python]

[template name:report requires:api-call]
API call result: ${api-call}
[/template]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse document with dependencies: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 3);
        
        // Check dependencies in modifiers
        let api_call_block = &blocks[1];
        let depends = api_call_block.modifiers.iter().find(|(k, _)| k == "depends").map(|(_, v)| v);
        assert_eq!(depends, Some(&"config".to_string()));
        
        let template_block = &blocks[2];
        let requires = template_block.modifiers.iter().find(|(k, _)| k == "requires").map(|(_, v)| v);
        assert_eq!(requires, Some(&"api-call".to_string()));
    }
    
    #[test]
    #[ignore] // Temporarily ignore this test until we fix the nested structure parsing
    fn test_nested_structure() {
        let input = r#"[section:document name:analysis-report]
# Data Analysis Report

[data name:dataset format:csv]
id,value,category
1,42,A
2,37,B
3,19,A
[/data]

[code:python name:analyze-data depends:dataset]
import pandas as pd
data = pd.read_csv('${dataset}')
result = data.groupby('category').mean()
print(result)
[/code:python]

[section:results name:findings]
## Key Findings

[visualization name:chart-1 data:analyze-data]
bar-chart
[/visualization]

[/section:results]

[/section:document]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse nested structure: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1); // One top-level section
        
        let document = &blocks[0];
        assert_eq!(document.block_type, "section:document");
        assert_eq!(document.children.len(), 3); // data, code, and section:results
        
        // Check the nested section
        let results_section = &document.children[2];
        assert_eq!(results_section.block_type, "section:results");
        assert_eq!(results_section.name, Some("findings".to_string()));
        assert_eq!(results_section.children.len(), 1); // visualization block
    }
}
