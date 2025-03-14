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
    fn test_nested_structure() {
        use yet_another_llm_project_but_better::parser::blocks::Block;
        
        // Create the document section directly
        let mut document_section = Block::new("section:document", Some("analysis-report"), "# Data Analysis Report");
        
        // Create the data block
        let data_content = "id,value,category\n1,42,A\n2,37,B\n3,19,A";
        let mut data_block = Block::new("data", Some("dataset"), data_content);
        data_block.add_modifier("format", "csv");
        
        // Create the code block
        let code_content = "import pandas as pd\ndata = pd.read_csv('${dataset}')\nresult = data.groupby('category').mean()\nprint(result)";
        let mut code_block = Block::new("code:python", Some("analyze-data"), code_content);
        code_block.add_modifier("depends", "dataset");
        
        // Create the nested results section
        let mut results_section = Block::new("section:results", Some("findings"), "## Key Findings");
        
        // Create the visualization block
        let mut viz_block = Block::new("visualization", Some("chart-1"), "bar-chart");
        viz_block.add_modifier("data", "analyze-data");
        
        // Add visualization to results section
        results_section.add_child(viz_block);
        
        // Add all blocks to the document section
        document_section.add_child(data_block);
        document_section.add_child(code_block);
        document_section.add_child(results_section);
        
        // Verify the structure
        assert_eq!(document_section.block_type, "section:document");
        assert_eq!(document_section.name, Some("analysis-report".to_string()));
        assert_eq!(document_section.children.len(), 3); // data, code, and section:results
        
        // Check the nested section
        let results_section = &document_section.children[2];
        assert_eq!(results_section.block_type, "section:results");
        assert_eq!(results_section.name, Some("findings".to_string()));
        assert_eq!(results_section.children.len(), 1); // visualization block
        
        // Check the visualization block
        let viz_block = &results_section.children[0];
        assert_eq!(viz_block.block_type, "visualization");
        assert_eq!(viz_block.name, Some("chart-1".to_string()));
        assert_eq!(viz_block.get_modifier("data"), Some(&"analyze-data".to_string()));
    }
}
