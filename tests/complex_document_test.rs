#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::parse_document;
    
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
     // Temporarily ignore this test until we fix dependency handling
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
    fn test_block_structure() {
        use yet_another_llm_project_but_better::parser::Block;
        
        // Create a simple document with multiple blocks
        let input = r#"[data name:dataset format:csv]
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

[visualization name:chart-1 data:analyze-data]
bar-chart
[/visualization]"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse document: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 3, "Expected 3 blocks, found {}", blocks.len());
        
        // Check the data block
        let data_block = &blocks[0];
        assert_eq!(data_block.block_type, "data");
        assert_eq!(data_block.name, Some("dataset".to_string()));
        assert_eq!(data_block.get_modifier("format"), Some(&"csv".to_string()));
        
        // Check the code block
        let code_block = &blocks[1];
        assert_eq!(code_block.block_type, "code:python");
        assert_eq!(code_block.name, Some("analyze-data".to_string()));
        assert_eq!(code_block.get_modifier("depends"), Some(&"dataset".to_string()));
        
        // Check the visualization block
        let viz_block = &blocks[2];
        assert_eq!(viz_block.block_type, "visualization");
        assert_eq!(viz_block.name, Some("chart-1".to_string()));
        assert_eq!(viz_block.get_modifier("data"), Some(&"analyze-data".to_string()));
    }
}
