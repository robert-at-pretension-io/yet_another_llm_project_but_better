#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::Block;
    
    #[test]
    fn test_template_declaration() {
        // Create a template block manually
        let mut block = Block::new("template", Some("api-request"), "");
        block.add_modifier("method", "GET");
        block.add_modifier("fallback", "api-fallback");
        
        block.content = r#"[code:javascript name:${endpoint}-request]
const response = await fetch('${base-url}/${endpoint}', {
  method: '${method}',
  headers: {
    'Authorization': 'Bearer ${api-key}',
    'Content-Type': 'application/json'
  }
});
const data = await response.json();
console.log(data);
[/code:javascript]"#.to_string();
        
        assert_eq!(block.block_type, "template");
        assert_eq!(block.name, Some("api-request".to_string()));
        
        assert_eq!(block.get_modifier("method"), Some(&"GET".to_string()));
        assert_eq!(block.get_modifier("fallback"), Some(&"api-fallback".to_string()));
        
        let content = block.content.as_str();
        assert!(content.contains("[code:javascript name:${endpoint}-request]"));
        assert!(content.contains("method: '${method}'"));
        assert!(content.contains("'${base-url}/${endpoint}'"));
    }
    
    #[test]
    fn test_template_invocation() {
        // Create template and invocation blocks manually
        let mut template_block = Block::new("template", Some("data-processor"), "");
        template_block.content = r#"[code:python name:process-${dataset-name}]
import pandas as pd

data = pd.read_csv('${dataset-path}')
processed = data.groupby('${group-by}').agg({
    '${value-col}': ['mean', 'sum', 'count']
})
print(processed)
[/code:python]"#.to_string();
        
        let mut invocation_block = Block::new("template_invocation", Some("data-processor"), "");
        invocation_block.add_modifier("dataset-name", "sales");
        invocation_block.add_modifier("dataset-path", "./data/sales.csv");
        invocation_block.add_modifier("group-by", "region");
        invocation_block.add_modifier("value-col", "revenue");
        
        let blocks = vec![template_block, invocation_block];
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, "template");
        assert_eq!(blocks[0].name, Some("data-processor".to_string()));
        
        // Check template invocation block
        assert_eq!(blocks[1].block_type, "template_invocation");
        assert_eq!(blocks[1].name, Some("data-processor".to_string()));
        assert_eq!(blocks[1].get_modifier("dataset-name"), Some(&"sales".to_string()));
        assert_eq!(blocks[1].get_modifier("dataset-path"), Some(&"./data/sales.csv".to_string()));
        assert_eq!(blocks[1].get_modifier("group-by"), Some(&"region".to_string()));
        assert_eq!(blocks[1].get_modifier("value-col"), Some(&"revenue".to_string()));
    }
    
    #[test]
    fn test_conditional_block() {
        // Create conditional blocks manually
        let mut prod_block = Block::new("conditional", None, "");
        prod_block.add_modifier("condition", "${env} === 'production'");
        prod_block.content = r#"[code:python name:prod-config]
DEBUG = False
DATABASE_URL = "postgresql://user:pass@prod-db:5432/app"
LOGGING_LEVEL = "WARNING"
[/code:python]"#.to_string();
        
        let mut dev_block = Block::new("conditional", None, "");
        dev_block.add_modifier("condition", "${env} === 'development'");
        dev_block.content = r#"[code:python name:dev-config]
DEBUG = True
DATABASE_URL = "postgresql://user:pass@localhost:5432/app_dev"
LOGGING_LEVEL = "DEBUG"
[/code:python]"#.to_string();
        
        let blocks = vec![prod_block, dev_block];
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, "conditional");
        assert_eq!(blocks[1].block_type, "conditional");
        
        assert_eq!(blocks[0].get_modifier("condition"), Some(&"${env} === 'production'".to_string()));
        assert_eq!(blocks[1].get_modifier("condition"), Some(&"${env} === 'development'".to_string()));
        
        assert!(blocks[0].content.contains("prod-config"));
        assert!(blocks[0].content.contains("prod-db:5432"));
        
        assert!(blocks[1].content.contains("dev-config"));
        assert!(blocks[1].content.contains("localhost:5432"));
    }
    
    #[test]
    fn test_error_block() {
        // Create error block manually
        let mut block = Block::new("error", None, "");
        block.add_modifier("type", "execution_failure");
        block.content = "Failed to execute 'data-processing' block. Runtime error: Division by zero.".to_string();
        
        assert_eq!(block.block_type, "error");
        assert_eq!(block.get_modifier("type"), Some(&"execution_failure".to_string()));
        assert!(block.content.contains("Failed to execute"));
        assert!(block.content.contains("Division by zero"));
    }
}
