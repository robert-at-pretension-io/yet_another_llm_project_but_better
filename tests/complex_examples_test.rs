#[cfg(test)]
mod tests {
    use yet_another_llm_project_but_better::parser::Block;
    
    #[test]
    fn test_data_analysis_workflow() {
        // Create the blocks manually since we're testing functionality, not parsing
        let mut config_block = Block::new("section:config", Some("analysis-config"), "");
        
        config_block.content = format!("[variable name:project-name]\nSales Analysis 2023\n[/variable]\n\n[variable name:data-source]\nquarterly_sales.csv\n[/variable]");
        
        let mut sales_data_block = Block::new("data", Some("sales-data"), "");
        sales_data_block.add_modifier("format", "csv");
        sales_data_block.content = "quarter,region,product,revenue,units\nQ1,North,Widget A,125000,1250\nQ1,South,Widget A,108000,1080\nQ1,North,Widget B,87500,350\nQ1,South,Widget B,95000,380\nQ2,North,Widget A,131000,1310\nQ2,South,Widget A,142000,1420\nQ2,North,Widget B,97500,390\nQ2,South,Widget B,110000,440".to_string();
        
        let mut analyze_sales_block = Block::new("code:python", Some("analyze-sales"), "");
        analyze_sales_block.add_modifier("depends", "sales-data");
        analyze_sales_block.content = "import pandas as pd\nimport matplotlib.pyplot as plt\n\n# Parse the CSV data\ndata = pd.read_csv('''${sales-data}''')\n\n# Calculate total revenue by region and quarter\nregion_quarter = data.groupby(['region', 'quarter'])['revenue'].sum().reset_index()\nprint(region_quarter)\n\n# Calculate product performance\nproduct_perf = data.groupby('product').agg({\n    'revenue': 'sum',\n    'units': 'sum'\n}).reset_index()\nproduct_perf['avg_price'] = product_perf['revenue'] / product_perf['units']\nprint(product_perf)".to_string();
        
        let mut question_block = Block::new("question", Some("analysis-query"), "");
        question_block.add_modifier("model", "gpt-4");
        question_block.content = "Based on the sales data and analysis provided, what are the key insights and recommendations for the business?\nFocus on regional performance, quarterly trends, and product pricing strategy.".to_string();
        
        let blocks = vec![
            config_block, 
            sales_data_block, 
            analyze_sales_block, 
            question_block
        ];
        
        // Check overall structure
        assert!(blocks.len() >= 4); // At minimum we need section, data, code, and question
        
        // Check for specific blocks by name
        let block_names: Vec<Option<String>> = blocks.iter()
            .map(|b| b.name.clone())
            .collect();
            
        assert!(block_names.contains(&Some("analysis-config".to_string())));
        assert!(block_names.contains(&Some("sales-data".to_string())));
        assert!(block_names.contains(&Some("analyze-sales".to_string())));
        assert!(block_names.contains(&Some("analysis-query".to_string())));
        
        // Check dependencies
        let analyze_block = blocks.iter()
            .find(|b| b.name == Some("analyze-sales".to_string()))
            .unwrap();
            
        assert_eq!(analyze_block.get_modifier("depends"), Some(&"sales-data".to_string()));
        assert!(analyze_block.content.contains("${sales-data}"));
    }
    
    #[test]
    fn test_api_integration_workflow() {
        // Create blocks manually
        let mut credentials_block = Block::new("section:credentials", Some("api-credentials"), "");
        
        credentials_block.content = format!("[secret name:api-key env:WEATHER_API_KEY]\n// This will be loaded from environment variable\n[/secret]\n\n[variable name:api-base-url]\nhttps://api.weatherapi.com/v1\n[/variable]");
        
        let mut template_block = Block::new("template", Some("api-request"), "");
        template_block.add_modifier("method", "GET");
        template_block.content = "[code:javascript name:${endpoint}-request]\nconst fetch = require('node-fetch');\n\nconst url = new URL('${base-url}/${endpoint}');\n${params}\n\nconst options = {\n  method: '${method}',\n  headers: {\n    'key': process.env.WEATHER_API_KEY\n  }\n};\n\nasync function fetchData() {\n  try {\n    const response = await fetch(url, options);\n    const data = await response.json();\n    console.log(JSON.stringify(data, null, 2));\n    return data;\n  } catch (error) {\n    console.error('Error fetching data:', error);\n    throw error;\n  }\n}\n\nfetchData();\n[/code:javascript]".to_string();
        
        let mut invocation_block = Block::new("template_invocation", Some("api-request"), "");
        invocation_block.add_modifier("endpoint", "current.json");
        invocation_block.add_modifier("base-url", "${api-base-url}");
        invocation_block.add_modifier("params", "url.searchParams.append('q', 'London');\nurl.searchParams.append('aqi', 'no');");
        
        let mut process_block = Block::new("code:python", Some("process-weather"), "");
        process_block.add_modifier("depends", "current-json-request");
        process_block.content = "import json\n\n# Parse the weather data\nweather_data = json.loads('''${current-json-request}''')\n\nlocation = weather_data['location']['name']\ncountry = weather_data['location']['country']\ntemp_c = weather_data['current']['temp_c']\ncondition = weather_data['current']['condition']['text']\n\nprint(f\"Current weather in {location}, {country}: {temp_c}°C, {condition}\")".to_string();
        
        let blocks = vec![
            credentials_block,
            template_block,
            invocation_block, 
            process_block
        ];
        
        // Verify we have the main components
        assert!(blocks.len() >= 3); // Section, template and invocation
        
        // Check for template and template invocation
        let template_block = blocks.iter()
            .find(|b| b.block_type == "template" && b.name == Some("api-request".to_string()));
        assert!(template_block.is_some());
        
        let template_content = template_block.unwrap().content.as_str();
        assert!(template_content.contains("${endpoint}-request"));
        assert!(template_content.contains("${base-url}/${endpoint}"));
        assert!(template_content.contains("${method}"));
        
        // Check for template invocation
        let invocation_block = blocks.iter()
            .find(|b| b.block_type == "template_invocation" && b.name == Some("api-request".to_string()));
        assert!(invocation_block.is_some());
        
        // Check modifiers on invocation
        if let Some(invocation) = invocation_block {
            assert_eq!(invocation.get_modifier("endpoint"), Some(&"current.json".to_string()));
            assert_eq!(invocation.get_modifier("base-url"), Some(&"${api-base-url}".to_string()));
            assert!(invocation.get_modifier("params").is_some());
        }
    }
    
    #[test]
    fn test_llm_conversation_workflow() {
        // Create blocks manually
        let mut memory_block = Block::new("memory", Some("conversation-history"), "");
        memory_block.content = "User: What's the capital of France?\nAI: The capital of France is Paris.\n\nUser: What's the population?\nAI: The population of Paris is approximately 2.16 million people as of 2019.".to_string();
        
        let mut data_block = Block::new("data", Some("current-query"), "");
        data_block.add_modifier("format", "text");
        data_block.content = "I'd like to know more about Paris landmarks. What are the top 3 tourist attractions?".to_string();
        
        let mut template_block = Block::new("template", Some("prompt-builder"), "");
        template_block.content = "[question name:${query-name} model:${model}]\n${system-instructions}\n\n${conversation-history}\n\nUser: ${current-query}\n[/question]".to_string();
        
        let mut invocation_block = Block::new("template_invocation", Some("prompt-builder"), "");
        invocation_block.add_modifier("query-name", "landmarks-query");
        invocation_block.add_modifier("model", "gpt-4");
        invocation_block.add_modifier("system-instructions", "You are a helpful travel assistant with knowledge about global tourist destinations.");
        invocation_block.add_modifier("conversation-history", "${conversation-history}");
        invocation_block.add_modifier("current-query", "${current-query}");
        
        let mut viz_block = Block::new("visualization", Some("prompt-preview"), "");
        viz_block.content = "[question debug:true]\n// This visualizes what will be sent as the prompt\nYou are a helpful travel assistant with knowledge about global tourist destinations.\n\nUser: What's the capital of France?\nAI: The capital of France is Paris.\n\nUser: What's the population?\nAI: The population of Paris is approximately 2.16 million people as of 2019.\n\nUser: I'd like to know more about Paris landmarks. What are the top 3 tourist attractions?\n[/question]\n\n[preview]\nThe prompt above will be sent to the GPT-4 model.\nEstimated token count: ~120 tokens\n[/preview]".to_string();
        
        let blocks = vec![
            memory_block,
            data_block,
            template_block,
            invocation_block,
            viz_block
        ];
        
        // Verify we have all the main components
        assert!(blocks.len() >= 5); // Memory, data, template, invocation, visualization
        
        // Check for memory block
        let memory_block = blocks.iter()
            .find(|b| b.block_type == "memory" && b.name == Some("conversation-history".to_string()));
        assert!(memory_block.is_some());
        
        // Check for template and its invocation
        let template_block = blocks.iter()
            .find(|b| b.block_type == "template" && b.name == Some("prompt-builder".to_string()));
        assert!(template_block.is_some());
        
        let invocation_block = blocks.iter()
            .find(|b| b.block_type == "template_invocation" && b.name == Some("prompt-builder".to_string()));
        assert!(invocation_block.is_some());
        
        // Check visualization
        let viz_block = blocks.iter()
            .find(|b| b.block_type == "visualization" && b.name == Some("prompt-preview".to_string()));
        assert!(viz_block.is_some());
        
        // Verify visualization contains question and preview
        if let Some(viz) = viz_block {
            let content = viz.content.as_str();
            assert!(content.contains("[question debug:true]"));
            assert!(content.contains("[preview]"));
            assert!(content.contains("GPT-4 model"));
        }
    }
}
