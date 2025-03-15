use crate::executor::MetaLanguageExecutor;
use crate::parser::{parse_document, Block};
use std::collections::HashMap;
use std::time::Duration;

#[cfg(test)]
mod advanced_tests {
    use super::*;

    /// Test complex nested sections with code blocks inside
    #[test]
    fn test_nested_sections_with_code() {
        let input = r#"
[section:analysis name:data-analysis]
# Data Analysis Workflow

[section:data-preparation]
## Data Preparation

[code:python name:generate-data]
import numpy as np
import pandas as pd

# Generate synthetic data
np.random.seed(42)
data = np.random.normal(0, 1, 1000)
df = pd.DataFrame({'values': data})
print(df.head().to_json())
[/code:python]

[code:python name:preprocess-data depends:generate-data]
import json
import pandas as pd

# Get data from previous step
raw_data = json.loads("""${generate-data}""")
df = pd.DataFrame(raw_data)

# Preprocess
df['normalized'] = (df['values'] - df['values'].mean()) / df['values'].std()
print(df.head().to_json())
[/code:python]
[/section:data-preparation]

[section:analysis]
## Analysis

[code:python name:analyze-data depends:preprocess-data]
import json
import pandas as pd
import numpy as np
from scipy import stats

# Get preprocessed data
data_json = """${preprocess-data}"""
df = pd.DataFrame(json.loads(data_json))

# Perform analysis
results = {
    'mean': float(df['values'].mean()),
    'median': float(df['values'].median()),
    'std': float(df['values'].std()),
    'skew': float(stats.skew(df['values'])),
    'kurtosis': float(stats.kurtosis(df['values']))
}

print(json.dumps(results, indent=2))
[/code:python]
[/section:analysis]

[section:visualization]
## Visualization

[code:python name:visualize-data depends:analyze-data cache_result:true timeout:30]
import json
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import io
import base64
from matplotlib.figure import Figure
from matplotlib.backends.backend_agg import FigureCanvasAgg as FigureCanvas

# Get analysis results
analysis_results = json.loads("""${analyze-data}""")

# Create a figure
fig = Figure(figsize=(10, 6))
canvas = FigureCanvas(fig)
ax = fig.add_subplot(111)

# Generate normal distribution with same parameters
x = np.linspace(-4, 4, 1000)
y = np.exp(-x**2/2) / np.sqrt(2*np.pi)
ax.plot(x, y, 'r-', lw=2, label='Normal Distribution')

# Plot histogram of our data
data_json = """${preprocess-data}"""
df = pd.DataFrame(json.loads(data_json))
ax.hist(df['values'], bins=30, density=True, alpha=0.6, label='Our Data')

# Add analysis results as text
textstr = '\n'.join((
    f"Mean: {analysis_results['mean']:.2f}",
    f"Median: {analysis_results['median']:.2f}",
    f"Std Dev: {analysis_results['std']:.2f}",
    f"Skewness: {analysis_results['skew']:.2f}",
    f"Kurtosis: {analysis_results['kurtosis']:.2f}"
))
props = dict(boxstyle='round', facecolor='wheat', alpha=0.5)
ax.text(0.05, 0.95, textstr, transform=ax.transAxes, fontsize=12,
        verticalalignment='top', bbox=props)

ax.set_xlabel('Value')
ax.set_ylabel('Density')
ax.set_title('Data Distribution Analysis')
ax.legend()
fig.tight_layout()

# Convert plot to base64 string
buf = io.BytesIO()
fig.savefig(buf, format='png')
buf.seek(0)
img_str = base64.b64encode(buf.read()).decode('utf-8')

print(f"data:image/png;base64,{img_str}")
[/code:python]
[/section:visualization]
[/section:analysis]

[section:api-workflow name:api-integration]
# API Integration Workflow

[data name:api-config]
{
  "base_url": "https://api.example.com",
  "endpoints": {
    "users": "/users",
    "posts": "/posts",
    "comments": "/comments"
  },
  "headers": {
    "Authorization": "Bearer ${api-token}",
    "Content-Type": "application/json"
  }
}
[/data]

[secret name:api-token]
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
[/secret]

[code:python name:prepare-api-request depends:api-config]
import json

# Load configuration
config = json.loads("""${api-config}""")

# Prepare request details
request = {
    "url": f"{config['base_url']}{config['endpoints']['users']}",
    "headers": config['headers'],
    "params": {
        "limit": 5
    }
}

print(json.dumps(request, indent=2))
[/code:python]

[api name:fetch-users depends:prepare-api-request]
${prepare-api-request.url}
[/api]

[code:python name:process-api-response depends:fetch-users]
import json

# Process the API response
try:
    response = json.loads("""${fetch-users}""")
    
    # Extract user information
    users = []
    for user in response:
        users.append({
            "id": user["id"],
            "name": user["name"],
            "email": user["email"]
        })
    
    print(json.dumps({"users": users}, indent=2))
except json.JSONDecodeError:
    print(json.dumps({"error": "Invalid API response", "raw": "${fetch-users}"}, indent=2))
[/code:python]

[code:python name:handle-api-error depends:process-api-response]
import json

# Check if there was an error in the previous step
response = json.loads("""${process-api-response}""")

if "error" in response:
    print(json.dumps({
        "status": "error",
        "message": "Failed to process API response",
        "details": response["error"]
    }, indent=2))
else:
    print(json.dumps({
        "status": "success",
        "message": f"Successfully processed {len(response['users'])} users",
        "data": response
    }, indent=2))
[/code:python]
[/section:api-workflow]

[section:ml-pipeline name:machine-learning]
# Machine Learning Pipeline

[code:python name:generate-ml-data]
import numpy as np
import pandas as pd
from sklearn.datasets import make_classification

# Generate a synthetic classification dataset
X, y = make_classification(
    n_samples=1000,
    n_features=20,
    n_informative=10,
    n_redundant=5,
    n_classes=2,
    random_state=42
)

# Convert to DataFrame
df = pd.DataFrame(X, columns=[f'feature_{i}' for i in range(X.shape[1])])
df['target'] = y

# Split into train/test
from sklearn.model_selection import train_test_split
train_df, test_df = train_test_split(df, test_size=0.2, random_state=42)

# Output as JSON
result = {
    "train": train_df.to_dict(orient='records'),
    "test": test_df.to_dict(orient='records'),
    "feature_names": [f'feature_{i}' for i in range(X.shape[1])],
    "target_name": "target"
}

# Print summary instead of full data to avoid huge output
summary = {
    "train_size": len(result["train"]),
    "test_size": len(result["test"]),
    "features": result["feature_names"],
    "target": result["target_name"]
}
print(json.dumps(summary, indent=2))
[/code:python]

[template name:model-training]
[code:python name:train-${model-name} depends:generate-ml-data]
import json
import numpy as np
import pandas as pd
from sklearn.metrics import accuracy_score, precision_score, recall_score, f1_score

# Get data summary
data_summary = json.loads("""${generate-ml-data}""")
feature_names = data_summary["features"]
target_name = data_summary["target"]

# In a real scenario, we would load the full data
# For this test, we'll generate new data with the same parameters
from sklearn.datasets import make_classification
X, y = make_classification(
    n_samples=1000,
    n_features=20,
    n_informative=10,
    n_redundant=5,
    n_classes=2,
    random_state=42
)

# Split into train/test
from sklearn.model_selection import train_test_split
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

# Train the model
from ${model-package} import ${model-class}
model = ${model-class}(${model-params})
model.fit(X_train, y_train)

# Evaluate
y_pred = model.predict(X_test)
metrics = {
    "accuracy": float(accuracy_score(y_test, y_pred)),
    "precision": float(precision_score(y_test, y_pred)),
    "recall": float(recall_score(y_test, y_pred)),
    "f1": float(f1_score(y_test, y_pred)),
    "model_type": "${model-name}"
}

print(json.dumps(metrics, indent=2))
[/code:python]
[/template]

[template:model-training name:random-forest model-name:random-forest model-package:sklearn.ensemble model-class:RandomForestClassifier model-params:n_estimators=100, random_state=42]
[/template:model-training]

[template:model-training name:logistic-regression model-name:logistic-regression model-package:sklearn.linear_model model-class:LogisticRegression model-params:random_state=42, max_iter=200]
[/template:model-training]

[template:model-training name:svm model-name:svm model-package:sklearn.svm model-class:SVC model-params:probability=True, random_state=42]
[/template:model-training]

[code:python name:compare-models depends:train-random-forest,train-logistic-regression,train-svm]
import json
import pandas as pd
import matplotlib.pyplot as plt
import io
import base64

# Get results from all models
rf_results = json.loads("""${train-random-forest}""")
lr_results = json.loads("""${train-logistic-regression}""")
svm_results = json.loads("""${train-svm}""")

# Combine results
all_results = [rf_results, lr_results, svm_results]
df_results = pd.DataFrame(all_results)

# Create comparison table
comparison = df_results[['model_type', 'accuracy', 'precision', 'recall', 'f1']]
comparison_html = comparison.to_html(index=False)

# Find best model
best_model = df_results.loc[df_results['f1'].idxmax()]
best_model_info = {
    "best_model": best_model["model_type"],
    "metrics": {
        "accuracy": best_model["accuracy"],
        "precision": best_model["precision"],
        "recall": best_model["recall"],
        "f1": best_model["f1"]
    },
    "comparison_table_html": comparison_html
}

print(json.dumps(best_model_info, indent=2))
[/code:python]
[/section:ml-pipeline]

[section:error-handling name:error-handling]
# Error Handling and Fallbacks

[code:python name:potentially-failing-code]
import random

# Simulate a failure with 50% probability
if random.random() < 0.5:
    raise Exception("Simulated failure")

print(json.dumps({"status": "success", "message": "Operation completed successfully"}))
[/code:python]

[data name:fallback-data]
{
  "status": "fallback",
  "message": "Using fallback data",
  "data": [1, 2, 3, 4, 5]
}
[/data]

[code:python name:process-with-fallback depends:potentially-failing-code fallback:fallback-data]
import json

try:
    # Try to parse the result from the previous step
    result = json.loads("""${potentially-failing-code}""")
    print(json.dumps({
        "source": "primary",
        "data": result
    }, indent=2))
except:
    # If there's an error, use the fallback data
    fallback = json.loads("""${fallback-data}""")
    print(json.dumps({
        "source": "fallback",
        "data": fallback
    }, indent=2))
[/code:python]

[conditional if:${process-with-fallback.source} == "fallback"]
[code:python name:handle-fallback-case depends:process-with-fallback]
import json

result = json.loads("""${process-with-fallback}""")
print(json.dumps({
    "status": "warning",
    "message": "Had to use fallback data",
    "fallback_data": result["data"]
}, indent=2))
[/code:python]
[/conditional]

[conditional if:${process-with-fallback.source} == "primary"]
[code:python name:handle-success-case depends:process-with-fallback]
import json

result = json.loads("""${process-with-fallback}""")
print(json.dumps({
    "status": "success",
    "message": "Used primary data source",
    "primary_data": result["data"]
}, indent=2))
[/code:python]
[/conditional]
[/section:error-handling]

[section:variable-resolution name:variable-resolution]
# Complex Variable Resolution

[data name:config]
{
  "app": {
    "name": "TestApp",
    "version": "1.0.0",
    "settings": {
      "debug": true,
      "timeout": 30,
      "retry_count": 3
    }
  },
  "database": {
    "host": "localhost",
    "port": 5432,
    "credentials": {
      "username": "test_user",
      "password": "${db-password}"
    }
  }
}
[/data]

[secret name:db-password]
super_secret_password
[/secret]

[variable name:app-name]${config.app.name}[/variable]
[variable name:app-version]${config.app.version}[/variable]
[variable name:debug-mode]${config.app.settings.debug}[/variable]
[variable name:db-connection-string]postgresql://${config.database.credentials.username}:${db-password}@${config.database.host}:${config.database.port}/testdb[/variable]

[code:python name:use-variables]
import json

variables = {
    "app_name": "${app-name}",
    "app_version": "${app-version}",
    "debug_mode": ${debug-mode},
    "connection_string": "${db-connection-string}"
}

print(json.dumps(variables, indent=2))
[/code:python]

[code:python name:nested-variable-resolution depends:use-variables]
import json

# Get variables from previous step
vars_json = """${use-variables}"""
variables = json.loads(vars_json)

# Create a message using these variables
message = f"""
Application: {variables['app_name']} v{variables['app_version']}
Debug Mode: {'Enabled' if variables['debug_mode'] else 'Disabled'}
Database: {variables['connection_string']}
"""

print(json.dumps({"message": message}, indent=2))
[/code:python]
[/section:variable-resolution]
"#;

        // Parse the document
        let blocks = parse_document(input).expect("Failed to parse document");
        
        // Verify the structure
        assert!(blocks.len() > 0, "No blocks were parsed");
        
        // Find the main sections
        let analysis_section = blocks.iter().find(|b| b.block_type == "section:analysis" && b.name.as_deref() == Some("data-analysis"));
        let api_section = blocks.iter().find(|b| b.block_type == "section:api-workflow" && b.name.as_deref() == Some("api-integration"));
        let ml_section = blocks.iter().find(|b| b.block_type == "section:ml-pipeline" && b.name.as_deref() == Some("machine-learning"));
        let error_section = blocks.iter().find(|b| b.block_type == "section:error-handling" && b.name.as_deref() == Some("error-handling"));
        let var_section = blocks.iter().find(|b| b.block_type == "section:variable-resolution" && b.name.as_deref() == Some("variable-resolution"));
        
        // Verify all main sections exist
        assert!(analysis_section.is_some(), "Analysis section not found");
        assert!(api_section.is_some(), "API workflow section not found");
        assert!(ml_section.is_some(), "Machine learning section not found");
        assert!(error_section.is_some(), "Error handling section not found");
        assert!(var_section.is_some(), "Variable resolution section not found");
        
        // Verify nested sections in analysis
        let analysis = analysis_section.unwrap();
        assert!(analysis.children.len() >= 3, "Analysis section should have at least 3 subsections");
        
        // Verify templates in ML section
        let ml = ml_section.unwrap();
        let templates = ml.children.iter().filter(|b| b.block_type.starts_with("template"));
        assert!(templates.count() >= 3, "ML section should have at least 3 templates");
    }

    /// Test complex dependency chains and execution
    #[test]
    fn test_complex_dependencies_execution() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a chain of dependent blocks
        let mut block_a = Block::new("data", Some("block-a"), "Initial data");
        executor.blocks.insert("block-a".to_string(), block_a.clone());
        executor.outputs.insert("block-a".to_string(), "Initial data".to_string());
        
        let mut block_b = Block::new("code:python", Some("block-b"), "print(f\"Processing: ${block-a}\")");
        block_b.add_modifier("depends", "block-a");
        executor.blocks.insert("block-b".to_string(), block_b.clone());
        
        let mut block_c = Block::new("code:python", Some("block-c"), "print(f\"Further processing: ${block-b}\")");
        block_c.add_modifier("depends", "block-b");
        executor.blocks.insert("block-c".to_string(), block_c.clone());
        
        let mut block_d = Block::new("code:python", Some("block-d"), "print(f\"Final result: ${block-c}\")");
        block_d.add_modifier("depends", "block-c");
        executor.blocks.insert("block-d".to_string(), block_d.clone());
        
        // Execute the final block, which should trigger all dependencies
        let result = executor.execute_block("block-d");
        assert!(result.is_ok(), "Execution failed: {:?}", result.err());
        
        // Verify all blocks were executed
        assert!(executor.outputs.contains_key("block-b"), "block-b was not executed");
        assert!(executor.outputs.contains_key("block-c"), "block-c was not executed");
        assert!(executor.outputs.contains_key("block-d"), "block-d was not executed");
    }

    /// Test circular dependency detection
    #[test]
    fn test_circular_dependency_detection() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create blocks with circular dependencies
        let mut block_x = Block::new("data", Some("block-x"), "Data with circular dependency: ${block-z}");
        executor.blocks.insert("block-x".to_string(), block_x.clone());
        
        let mut block_y = Block::new("code:python", Some("block-y"), "print(f\"Middle block: ${block-x}\")");
        block_y.add_modifier("depends", "block-x");
        executor.blocks.insert("block-y".to_string(), block_y.clone());
        
        let mut block_z = Block::new("code:python", Some("block-z"), "print(f\"Circular reference: ${block-y}\")");
        block_z.add_modifier("depends", "block-y");
        executor.blocks.insert("block-z".to_string(), block_z.clone());
        
        // Execute should detect the circular dependency
        let result = executor.execute_block("block-z");
        assert!(result.is_err(), "Circular dependency not detected");
        
        match result {
            Err(crate::executor::ExecutorError::CircularDependency(_)) => {
                // This is the expected error
            },
            _ => panic!("Wrong error type returned for circular dependency")
        }
    }

    /// Test fallback mechanism
    #[test]
    fn test_fallback_mechanism() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a block that will fail
        let failing_block = Block::new("code:python", Some("failing-block"), 
                                      "raise Exception(\"This block always fails\")");
        executor.blocks.insert("failing-block".to_string(), failing_block);
        
        // Create a fallback block
        let fallback_block = Block::new("data", Some("fallback-data"), 
                                       "{\"status\": \"fallback\", \"message\": \"Using fallback data\"}");
        executor.blocks.insert("fallback-data".to_string(), fallback_block.clone());
        executor.outputs.insert("fallback-data".to_string(), 
                               "{\"status\": \"fallback\", \"message\": \"Using fallback data\"}".to_string());
        
        // Create a block that depends on the failing block but has a fallback
        let mut dependent_block = Block::new("code:python", Some("dependent-block"), 
                                           "print(f\"Got data: ${failing-block}\")");
        dependent_block.add_modifier("depends", "failing-block");
        dependent_block.add_modifier("fallback", "fallback-data");
        executor.blocks.insert("dependent-block".to_string(), dependent_block);
        
        // Register the fallback
        executor.fallbacks.insert("failing-block".to_string(), "fallback-data".to_string());
        
        // Execute the dependent block
        let result = executor.execute_block("dependent-block");
        
        // It should succeed using the fallback
        assert!(result.is_ok(), "Execution with fallback failed: {:?}", result.err());
        
        // The output should contain the fallback data
        let output = result.unwrap();
        assert!(output.contains("fallback"), 
                "Output doesn't contain fallback data: {}", output);
    }

    /// Test variable resolution across section boundaries
    #[test]
    fn test_cross_section_variable_resolution() {
        let input = r#"
[section:outer name:outer-section]
[data name:outer-data]
{
  "value": "outer value"
}
[/data]

[section:inner name:inner-section]
[data name:inner-data]
{
  "value": "inner value",
  "outer_reference": "${outer-data.value}"
}
[/data]

[code:python name:process-data]
import json

inner_data = json.loads("""${inner-data}""")
print(f"Inner value: {inner_data['value']}")
print(f"Outer reference: {inner_data['outer_reference']}")
[/code:python]
[/section:inner]
[/section:outer]
"#;

        // Parse the document
        let blocks = parse_document(input).expect("Failed to parse document");
        
        // Create executor and add all blocks
        let mut executor = MetaLanguageExecutor::new();
        
        // Find and add the outer data block
        let outer_section = blocks.iter().find(|b| b.block_type == "section:outer").unwrap();
        let outer_data = outer_section.children.iter().find(|b| b.name.as_deref() == Some("outer-data")).unwrap();
        executor.blocks.insert("outer-data".to_string(), outer_data.clone());
        executor.outputs.insert("outer-data".to_string(), outer_data.content.clone());
        
        // Find and add the inner section and its blocks
        let inner_section = outer_section.children.iter().find(|b| b.block_type == "section:inner").unwrap();
        
        // Add inner data block
        let inner_data = inner_section.children.iter().find(|b| b.name.as_deref() == Some("inner-data")).unwrap();
        executor.blocks.insert("inner-data".to_string(), inner_data.clone());
        
        // Add process-data block
        let process_block = inner_section.children.iter().find(|b| b.name.as_deref() == Some("process-data")).unwrap();
        executor.blocks.insert("process-data".to_string(), process_block.clone());
        
        // Execute the inner data block first to resolve variables
        let inner_result = executor.execute_block("inner-data");
        assert!(inner_result.is_ok(), "Failed to execute inner-data block");
        
        // Now execute the process block
        let result = executor.execute_block("process-data");
        assert!(result.is_ok(), "Failed to execute process-data block");
        
        // Verify cross-section reference was resolved
        let output = result.unwrap();
        assert!(output.contains("outer value"), 
                "Output doesn't contain the outer section value: {}", output);
    }

    /// Test caching mechanism
    #[test]
    fn test_caching_mechanism() {
        let mut executor = MetaLanguageExecutor::new();
        
        // Create a block with caching enabled
        let mut cached_block = Block::new("code:python", Some("cached-block"), 
                                        "import time\ntime.sleep(0.1)\nprint(\"Executed at\", time.time())");
        cached_block.add_modifier("cache_result", "true");
        cached_block.add_modifier("timeout", "10"); // 10 second cache timeout
        executor.blocks.insert("cached-block".to_string(), cached_block);
        
        // Execute the block first time
        let first_result = executor.execute_block("cached-block");
        assert!(first_result.is_ok(), "First execution failed");
        
        // Execute again immediately - should use cache
        let second_result = executor.execute_block("cached-block");
        assert!(second_result.is_ok(), "Second execution failed");
        
        // Results should be identical
        assert_eq!(first_result.unwrap(), second_result.unwrap(), 
                  "Cached result differs from original");
        
        // Verify it's in the cache
        assert!(executor.cache.contains_key("cached-block"), 
                "Block not found in cache");
    }

    /// Test template instantiation and execution
    #[test]
    fn test_template_instantiation() {
        let input = r#"
[template name:data-processor]
[code:python name:process-${data-name}]
import json

data = json.loads("""${${data-name}}""")
result = {
    "processed": True,
    "source": "${data-name}",
    "values": data
}
print(json.dumps(result, indent=2))
[/code:python]
[/template]

[data name:sample-data-1]
{
  "id": 1,
  "name": "Sample 1"
}
[/data]

[data name:sample-data-2]
{
  "id": 2,
  "name": "Sample 2"
}
[/data]

[template:data-processor data-name:sample-data-1]
[/template:data-processor]

[template:data-processor data-name:sample-data-2]
[/template:data-processor]

[code:python name:combine-results depends:process-sample-data-1,process-sample-data-2]
import json

result1 = json.loads("""${process-sample-data-1}""")
result2 = json.loads("""${process-sample-data-2}""")

combined = {
    "results": [result1, result2]
}
print(json.dumps(combined, indent=2))
[/code:python]
"#;

        // Parse the document
        let blocks = parse_document(input).expect("Failed to parse document");
        
        // Create executor
        let mut executor = MetaLanguageExecutor::new();
        
        // Add all blocks to the executor
        for block in &blocks {
            if let Some(name) = &block.name {
                executor.blocks.insert(name.clone(), block.clone());
                
                // Add data blocks directly to outputs
                if block.block_type == "data" {
                    executor.outputs.insert(name.clone(), block.content.clone());
                }
            }
            
            // Process template instantiations
            if block.block_type.starts_with("template:") {
                // In a real implementation, this would create the instantiated blocks
                // For this test, we'll assume the parser has already created them
                
                // Find the instantiated blocks in the parsed blocks
                if let Some(data_name) = block.get_modifier("data-name") {
                    let process_name = format!("process-{}", data_name);
                    
                    // Find the instantiated block
                    if let Some(instantiated) = blocks.iter().find(|b| b.name.as_deref() == Some(&process_name)) {
                        executor.blocks.insert(process_name, instantiated.clone());
                    }
                }
            }
        }
        
        // Add the combine-results block
        if let Some(combine_block) = blocks.iter().find(|b| b.name.as_deref() == Some("combine-results")) {
            executor.blocks.insert("combine-results".to_string(), combine_block.clone());
        }
        
        // Execute the combine-results block
        let result = executor.execute_block("combine-results");
        assert!(result.is_ok(), "Failed to execute combined results: {:?}", result.err());
        
        // Verify the output contains both processed results
        let output = result.unwrap();
        assert!(output.contains("Sample 1"), "Output missing Sample 1 data");
        assert!(output.contains("Sample 2"), "Output missing Sample 2 data");
    }
}
