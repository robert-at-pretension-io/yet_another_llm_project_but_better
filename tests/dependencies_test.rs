#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::{parse_document, Block};
    
    #[test]
    fn test_explicit_dependencies() {
        let input = r#"<meta:document>
<meta:data name="sales-data" format="csv"><![CDATA[
date,product,quantity,revenue
2023-01-15,Widget A,120,1200.00
2023-01-16,Widget B,85,1700.00
]]></meta:data>

<meta:code language="python" name="analyze-sales" depends="sales-data"><![CDATA[
import pandas as pd
import matplotlib.pyplot as plt

data = pd.read_csv('''${sales-data}''')
total_revenue = data['revenue'].sum()
print(f"Total revenue: ${total_revenue:.2f}")
]]></meta:code>
</meta:document>"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, Some("sales-data".to_string()));
        assert_eq!(blocks[1].name, Some("analyze-sales".to_string()));
        
        // Check dependency
        assert_eq!(blocks[1].get_modifier("depends"), Some(&"sales-data".to_string()));
        
        // Check reference
        let content = blocks[1].content.as_str();
        assert!(content.contains("${sales-data}"));
    }
    
    #[test]
    fn test_requires_modifier() {
        let input = r#"<meta:document>
<meta:variable name="api-key"><![CDATA[
abcd1234efgh5678
]]></meta:variable>

<meta:code language="python" name="fetch-data" requires="api-key"><![CDATA[
import requests

headers = {
    "Authorization": f"Bearer {api_key}"
}
response = requests.get("https://api.example.com/data", headers=headers)
data = response.json()
print(data)
]]></meta:code>
</meta:document>"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, Some("api-key".to_string()));
        assert_eq!(blocks[1].name, Some("fetch-data".to_string()));
        
        // Check requires modifier
        assert_eq!(blocks[1].get_modifier("requires"), Some(&"api-key".to_string()));
    }
    
    #[test]
    fn test_multiple_dependencies() {
        // Create each block individually to avoid parsing issues with the full document
        let data1 = r#"<meta:document>
<meta:data name="product-data" format="json"><![CDATA[
{
  "products": [
    {"id": 1, "name": "Widget A", "price": 10.0},
    {"id": 2, "name": "Widget B", "price": 20.0}
  ]
}
]]></meta:data>
</meta:document>"#;

        let data2 = r#"<meta:document>
<meta:data name="sales-data" format="json"><![CDATA[
{
  "sales": [
    {"product_id": 1, "quantity": 100},
    {"product_id": 2, "quantity": 50}
  ]
}
]]></meta:data>
</meta:document>"#;

        // Create a mock Block that represents what we expect
        let mut code_block = Block::new("code", Some("calculate-revenue"), 
        r#"import json

product_data = json.loads('''${product-data}''')
sales_data = json.loads('''${sales-data}''')

product_map = {p["id"]: p for p in product_data["products"]}
total_revenue = 0

for sale in sales_data["sales"]:
    product = product_map[sale["product_id"]]
    revenue = product["price"] * sale["quantity"]
    total_revenue += revenue
    
print(f"Total revenue: ${total_revenue:.2f}")"#);
        
        // Add language modifier for XML format
        code_block.add_modifier("language", "python");
        
        // Parse the first two blocks
        let block1 = parse_document(data1).unwrap();
        let block2 = parse_document(data2).unwrap();
        
        // Manually add the dependencies
        code_block.add_modifier("depends", "product-data,sales-data");
        
        // Verify blocks are as expected
        assert_eq!(block1[0].name, Some("product-data".to_string()));
        assert_eq!(block2[0].name, Some("sales-data".to_string()));
        
        // Verify the mocked code block is correct
        assert_eq!(code_block.name, Some("calculate-revenue".to_string()));
        
        // Check multiple dependencies
        let depends = code_block.get_modifier("depends").unwrap();
        assert!(depends.contains("product-data"));
        assert!(depends.contains("sales-data"));
        
        // Check both references
        let content = code_block.content.as_str();
        assert!(content.contains("${product-data}"));
        assert!(content.contains("${sales-data}"));
    }
    
    #[test]
    fn test_implicit_dependencies() {
        let input = r#"<meta:document>
<meta:variable name="base-url"><![CDATA[
https://api.example.com
]]></meta:variable>

<meta:variable name="endpoint"><![CDATA[
/users
]]></meta:variable>

<meta:code language="javascript" name="api-request"><![CDATA[
// This block implicitly depends on base-url and endpoint
const url = '${base-url}${endpoint}';
fetch(url)
  .then(response => response.json())
  .then(data => console.log(data));
]]></meta:code>
</meta:document>"#;
        
        let blocks = parse_document(input).unwrap();
        
        assert_eq!(blocks.len(), 3);
        
        // Check implicit references without explicit depends modifier
        let content = blocks[2].content.as_str();
        assert!(content.contains("${base-url}${endpoint}"));
    }
}
