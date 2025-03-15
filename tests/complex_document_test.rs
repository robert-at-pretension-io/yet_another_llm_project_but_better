#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::parse_document;
    
    #[test]
    fn test_mixed_block_types() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="user-info" format="json">
  <![CDATA[
{"name": "Alice", "role": "Developer"}
  ]]>
  </meta:data>

  <meta:code language="python" name="greet-user">
  <![CDATA[
import json
user = json.loads('${user-info}')
print(f"Hello, {user['name']}! You are a {user['role']}.")
  ]]>
  </meta:code>

  <meta:shell name="run-script">
  <![CDATA[
python script.py
  ]]>
  </meta:shell>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse mixed blocks: {:?}", result.err());
        
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 3, "Expected 3 blocks, found {}", blocks.len());
        
        assert_eq!(blocks[0].block_type, "data");
        assert_eq!(blocks[1].block_type, "code:python");
        assert_eq!(blocks[2].block_type, "shell");
    }
    
    #[test]
    // Test for dependency handling
    fn test_document_with_dependencies() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="config" format="json">
  <![CDATA[
{"api_url": "https://api.example.com", "timeout": 30}
  ]]>
  </meta:data>

  <meta:code language="python" name="api-call" depends="config">
  <![CDATA[
import json
import requests

config = json.loads('${config}')
response = requests.get(config['api_url'], timeout=config['timeout'])
print(response.status_code)
  ]]>
  </meta:code>

  <meta:template name="report" requires="api-call">
  <![CDATA[
API call result: ${api-call}
  ]]>
  </meta:template>
</meta:document>"#;
        
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
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:data name="dataset" format="csv">
  <![CDATA[
id,value,category
1,42,A
2,37,B
3,19,A
  ]]>
  </meta:data>

  <meta:code language="python" name="analyze-data" depends="dataset">
  <![CDATA[
import pandas as pd
data = pd.read_csv('${dataset}')
result = data.groupby('category').mean()
print(result)
  ]]>
  </meta:code>

  <meta:visualization name="chart-1" data="analyze-data">
  <![CDATA[
bar-chart
  ]]>
  </meta:visualization>
</meta:document>"#;
        
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
    
    #[test]
    fn test_nested_structure() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<meta:document xmlns:meta="https://example.com/meta-language">
  <meta:section type="document" name="analysis-report">
  <![CDATA[
# Data Analysis Report
  ]]>
  
    <meta:data name="dataset" format="csv">
    <![CDATA[
id,value,category
1,42,A
2,37,B
3,19,A
    ]]>
    </meta:data>

    <meta:code language="python" name="analyze-data" depends="dataset">
    <![CDATA[
import pandas as pd
data = pd.read_csv('${dataset}')
result = data.groupby('category').mean()
print(result)
    ]]>
    </meta:code>

    <meta:section type="results" name="data-results">
    <![CDATA[
## Results
The analysis of the data shows interesting patterns.
    ]]>
    </meta:section>
    
  </meta:section>
</meta:document>"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Failed to parse nested structure: {:?}", result.err());
        
        let blocks = result.unwrap();
        
        // Debug the parsed blocks
        println!("Number of blocks: {}", blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            println!("Block {}: type='{}', name={:?}", i, block.block_type, block.name);
            println!("  Modifiers: {:?}", block.modifiers);
            println!("  Children count: {}", block.children.len());
            for (j, child) in block.children.iter().enumerate() {
                println!("    Child {}: type='{}', name={:?}", j, child.block_type, child.name);
            }
        }
        
        // The XML parser properly handles nested blocks
        assert_eq!(blocks.len(), 1, "Expected 1 top-level block, found {}", blocks.len());
        
        // Check the top-level section block
        let section_block = &blocks[0];
        assert_eq!(section_block.name, Some("analysis-report".to_string()));
        assert_eq!(section_block.block_type, "section:document");
        assert!(section_block.content.contains("# Data Analysis Report"));
        
        // Check that it has 3 children
        assert_eq!(section_block.children.len(), 3, "Expected 3 child blocks, found {}", section_block.children.len());
        
        // Check the data block (first child)
        let dataset_block = &section_block.children[0];
        assert_eq!(dataset_block.name, Some("dataset".to_string()));
        assert_eq!(dataset_block.block_type, "data");
        assert!(dataset_block.content.contains("id,value,category"));
        
        // Check the code block (second child)
        let code_block = &section_block.children[1];  
        assert_eq!(code_block.name, Some("analyze-data".to_string()));
        assert_eq!(code_block.block_type, "code:python");
        assert_eq!(code_block.get_modifier("depends"), Some(&"dataset".to_string()));
        
        // Check the section block (third child)
        let results_section = &section_block.children[2];
        assert_eq!(results_section.name, Some("data-results".to_string()));
        assert_eq!(results_section.block_type, "section:results");
        assert!(results_section.content.contains("## Results"));
        assert!(results_section.content.contains("interesting patterns"));
    }
}
