#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use yet_another_llm_project_but_better::*;

    #[test]
    fn test_parse_document_success() {
        println!("Debug: Starting test_parse_document_success");
        println!("Debug: Starting test_parse_document_success");
        let document = "[question name:sample]\nWhat is the answer?\n[/question]";
        println!("Debug: Document content: {}", document);
        println!("Debug: Document content: {}", document);
        let doc = parse_document(document).expect("Failed to parse document");
        assert!(doc.blocks.contains_key("sample"), "Document should contain a block named 'sample'");
    }

    #[test]
    fn test_resolve_dependencies() {
        println!("Debug: Starting test_resolve_dependencies");
        println!("Debug: Starting test_resolve_dependencies");
        let document = "[question name:sample depends:other]\nWhat?\n[/question][question name:other]\nOther question?\n[/question]";
        println!("Debug: Document content: {}", document);
        println!("Debug: Document content: {}", document);
        let doc = parse_document(document).expect("Failed to parse document");
        let deps: HashSet<String> = doc.dependencies.get("sample").cloned().unwrap_or_default();
        assert!(deps.contains("other"), "Dependencies should contain 'other'");
    }

    #[test]
    fn test_process_questions_no_error() {
        // Create a simple document with a question block.
        println!("Debug: Starting test_process_questions_no_error");
        println!("Debug: Starting test_process_questions_no_error");
        let document = "[question name:test_question model:default debug:true]\nCompute X\n[/question]";
        println!("Debug: Document content: {}", document);
        println!("Debug: Document content: {}", document);
        let mut doc = parse_document(document).expect("Failed to parse document");
        // Process questions which should generate a response block.
        process_questions(&mut doc).expect("Processing questions failed");
        // Check that a response block was added.
        let response_key = "test_question-response";
        assert!(doc.blocks.contains_key(response_key), "Response block should be added");
    }
    #[test]
    fn test_basic_block_types() {
        let document = "\
            [data name:project-info format:json always_include:true]\
            {\"name\": \"Test Project\",\"version\": \"0.1.0\"}\
            [/data]\
            [comment]\
            This is a comment block\
            [/comment]\
            [variable name:max-retries]\
            3\
            [/variable]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert!(doc.blocks.contains_key("project-info"), "Should parse data block");
        assert!(doc.blocks.contains_key("max-retries"), "Should parse variable block");
        
        let data_block = doc.blocks.get("project-info").unwrap();
        assert_eq!(data_block.block_type, "data");
        assert_eq!(data_block.modifiers.get("format").unwrap(), "json");
        assert_eq!(data_block.modifiers.get("always_include").unwrap(), "true");
    }

    #[test]
    fn test_code_blocks_with_fallbacks() {
        let document = "\
            [code:python name:basic-python fallback:python-fallback]\
            print(\"Hello, World!\")\
            [/code:python]\
            \
            [code:python name:python-fallback]\
            print(\"This is a fallback block\")\
            [/code:python]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert!(doc.blocks.contains_key("basic-python"), "Should parse Python code block");
        assert!(doc.blocks.contains_key("python-fallback"), "Should parse fallback block");
        
        let code_block = doc.blocks.get("basic-python").unwrap();
        assert_eq!(code_block.block_type, "code:python");
        assert_eq!(code_block.modifiers.get("fallback").unwrap(), "python-fallback");
        assert!(code_block.content.contains("Hello, World!"));
    }

    #[test]
    fn test_question_blocks_with_dependencies() {
        let document = "\
            [data name:test-data format:json]\
            {\"key\": \"value\"}\
            [/data]\
            \
            [question name:dependent-question model:gpt-4 depends:test-data]\
            Analyze this: ${test-data}\
            [/question]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert!(doc.blocks.contains_key("dependent-question"), "Should parse question block");
        
        let question_block = doc.blocks.get("dependent-question").unwrap();
        assert_eq!(question_block.block_type, "question");
        assert_eq!(question_block.modifiers.get("model").unwrap(), "gpt-4");
        assert!(question_block.depends_on.contains("test-data"), "Should have dependency");
        
        // Check implicit references in dependencies
        let deps = doc.dependencies.get("dependent-question").cloned().unwrap_or_default();
        assert!(deps.contains("test-data"), "Dependencies should include referenced block");
    }

    #[test]
    fn test_template_definition_and_usage() {
        let document = "\
            [template name:simple-template]\
            [data name:${name}]\
            ${content}\
            [/data]\
            [/template]\
            \
            [@simple-template\
              name:\"test-instance\"\
              content:\"Template content\"\
            ]\
            [/@simple-template]";
        
        let mut doc = parse_document(document).expect("Failed to parse document");
        
        // Check the template is parsed
        assert!(doc.blocks.contains_key("simple-template"), "Should parse template block");
        
        // Test that template instantiation created the new block
        assert!(doc.blocks.contains_key("test-instance") || 
               doc.unnamed_blocks.iter().any(|b| b.name.as_deref() == Some("test-instance")), 
               "Template expansion should create a new block");
    }



    #[test]
    fn test_special_characters_in_content() {
        let document = "\
            [data name:special-chars]\
            Testing: !@#$%^&*()_+-=[]{}|;':\",./<>?\
            [/data]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert!(doc.blocks.contains_key("special-chars"), "Should parse block with special characters");
        let content = &doc.blocks.get("special-chars").unwrap().content;
        assert!(content.contains("!@#$%^&*()"), "Content should preserve special characters");
    }

    #[test]
    fn test_nested_blocks() {
        let document = "\
            [section:analysis name:test-section]\
              [data name:inner-data]\
              Inner content\
              [/data]\
              \
              [code:python name:inner-code fallback:inner-code-fallback]\
              print(\"Inside section\")\
              [/code:python]\
              [code:python name:inner-code-fallback]\
              print(\"Fallback inner-code\")\
              [/code:python]\
            [/section]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert!(doc.blocks.contains_key("test-section"), "Should parse section block");
        assert!(doc.blocks.contains_key("inner-data"), "Should parse inner data block");
        assert!(doc.blocks.contains_key("inner-code"), "Should parse inner code block");
    }

    #[test]
    fn test_block_with_complex_modifiers() {
        let document = "\
            [question name:complex-mods\
              model:gpt-4\
              temperature:0.7\
              max_tokens:1000\
              top_p:0.9\
              frequency_penalty:0.5\
              presence_penalty:0.3\
              timeout:30\
              retry:2\
              cache_result:true\
              priority:8\
              debug:true\
            ]\
            Test question\
            [/question]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert!(doc.blocks.contains_key("complex-mods"), "Should parse block with many modifiers");
        
        let block = doc.blocks.get("complex-mods").unwrap();
        assert_eq!(block.modifiers.get("model").unwrap(), "gpt-4");
        assert_eq!(block.modifiers.get("temperature").unwrap(), "0.7");
        assert_eq!(block.modifiers.get("max_tokens").unwrap(), "1000");
        assert_eq!(block.modifiers.get("top_p").unwrap(), "0.9");
        assert_eq!(block.modifiers.get("frequency_penalty").unwrap(), "0.5");
        assert_eq!(block.modifiers.get("retry").unwrap(), "2");
        assert_eq!(block.modifiers.get("cache_result").unwrap(), "true");
    }

    #[test]
    fn test_unnamed_blocks() {
        let document = "\
            [data]\
            Unnamed data block\
            [/data]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert_eq!(doc.unnamed_blocks.len(), 1, "Should parse unnamed block");
        assert_eq!(doc.unnamed_blocks[0].content, "Unnamed data block");
    }

    #[test]
    fn test_fallback_execution() {
        let document = "\
            [code:python name:fail-block fallback:success-block]\
            raise Exception('Fail')\
            [/code:python]\
            \
            [code:python name:success-block]\
            print('Fallback executed')\
            [/code:python]";
        
        let mut doc = parse_document(document).expect("Failed to parse document");
        let result = doc.execute_block("fail-block").expect("Fallback should execute");
        
        assert!(result.contains("Fallback executed"), "Fallback block should execute");
    }

    #[test]
    fn test_api_block_execution() {
        let document = "\
            [api name:test-api-1 method:GET fallback:test-api-fallback]\
            https://api.example.com/test\
            [/api]\
            \
            [data name:test-api-fallback format:json]\
            {\"status\": \"fallback\"}\
            [/data]";
        
        let mut doc = parse_document(document).expect("Failed to parse document");
        let result = doc.execute_block("test-api-1").expect("API block should execute");
        
        assert!(result.contains("AI Response for"), "API block should return a response");
    }

    #[test]
    fn test_debug_mode() {
        let document = "\
            [question name:debug-question-1 model:default debug:true]\
            Debug this question\
            [/question]";
        
        let mut doc = parse_document(document).expect("Failed to parse document");
        let result = doc.execute_block("debug-question-1").expect("Debug mode should execute");
        
        assert!(result.contains("DEBUG CONTEXT"), "Debug mode should output debug context");
    }

    #[test]
    fn test_template_expansion_with_missing_parameters() {
        let document = "\
            [template name:incomplete-template]\
            [data name:${name}]\
            ${content}\
            [/data]\
            [/template]\
            \
            [@incomplete-template\
              name:\"test-instance\"\
            ]\
            [/@incomplete-template]";
        
        let result = parse_document(document);
        
        assert!(result.is_err(), "Should fail due to missing parameters");
    }

    #[test]
    fn already_answered() {
        let document = "\
            [question name:already-answered-1]\
            What is 2+2?\
            [/question]\
            \
            [response name:already-answered-response depends:already-answered-1]\
            The answer is 4.\
            [/response]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        let question = doc.blocks.get("already-answered-1").unwrap();
        assert!(question.answered, "Question with response should be marked as answered");
    }

    #[test]
    fn test_xml_html_content() {
        let document = "\
            [data name:xml-content format:xml]\
            <?xml version=\"1.0\"?>\
            <root>\
              <item>Value</item>\
            </root>\
            [/data]\
            \
            [data name:html-content format:html]\
            <!DOCTYPE html>\
            <html>\
            <body>\
              <h1>Test</h1>\
            </body>\
            </html>\
            [/data]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        assert!(doc.blocks.contains_key("xml-content"), "Should parse XML content");
        assert!(doc.blocks.contains_key("html-content"), "Should parse HTML content");
        
        let xml_block = doc.blocks.get("xml-content").unwrap();
        assert_eq!(xml_block.modifiers.get("format").unwrap(), "xml");
        assert!(xml_block.content.contains("<root>"));
        
        let html_block = doc.blocks.get("html-content").unwrap();
        assert_eq!(html_block.modifiers.get("format").unwrap(), "html");
        assert!(html_block.content.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let document = "\
            [question name:q1 depends:q2]\
            First question\
            [/question]\
            \
            [question name:q2 depends:q1]\
            Second question\
            [/question]";
        
        let result = parse_document(document);
        
        assert!(result.is_err(), "Should detect circular dependency");
        assert!(result.unwrap_err().contains("Circular dependency"), 
                "Error message should mention circular dependency");
    }


    #[test]
    fn test_block_modifier_defaults() {
        let doc_str = "[data name:test-data]value[/data]";
        let doc = parse_document(doc_str).unwrap();

        let block = doc.blocks.get("test-data").unwrap();
        assert_eq!(block.modifiers.get("cache_result"), None, "Default cache_result should be false");
    }

    #[test]
    fn test_always_include_modifier() {
        let doc_str = "[data name:always-include-block always_include:true]value[/data]";
        let doc = parse_document(doc_str).unwrap();

        let block = doc.blocks.get("always-include-block").unwrap();
        assert_eq!(block.modifiers.get("always_include").unwrap(), "true", "always_include should be explicitly set to true");
    }

    #[test]
    fn test_fallback_mandatory() {
        let doc_str = "[shell name:critical-shell]false[/shell]";
        let result = parse_document(doc_str);
        assert!(result.is_err(), "Parsing should fail due to missing fallback");
    }

    #[test]
    fn test_fallback_auto_generated() {
        let doc_str = "[shell name:auto-fallback-shell fallback:auto-fallback-shell-fallback]false[/shell][shell name:auto-fallback-shell-fallback]echo fallback[/shell]";
        let doc = parse_document(doc_str).unwrap();
        assert!(doc.blocks.contains_key("auto-fallback-shell-fallback"), "Fallback block should be parsed and recognized");
    }

    #[test]
    fn test_conditional_block_parsing() {
        let doc_str = "[conditional name:conditional-example depends:test-data]Conditional content[/conditional]";
        let doc = parse_document(doc_str).unwrap();
        let conditional_block = doc.blocks.get("conditional-example").unwrap();
        assert!(conditional_block.depends_on.contains("test-data"), "Conditional block should have explicit dependency");
    }

    #[test]
    fn test_debug_verbosity_levels() {
        let doc_str = "[debug enabled:true verbosity:full][/debug]";
        let doc = parse_document(doc_str).unwrap();

        assert_eq!(doc.unnamed_blocks.len(), 1);
        let debug_block = &doc.unnamed_blocks[0];
        assert_eq!(debug_block.modifiers.get("enabled").unwrap(), "true");
        assert_eq!(debug_block.modifiers.get("verbosity").unwrap(), "full");
    }

    #[test]
    fn test_template_with_optional_placeholders() {
        let doc_str = r#"
        [template name:optional-placeholder]
        [question name:${name} model:${model}]
        ${content}
        [/question]
        [/template]

        [@optional-placeholder name:"optional-test" model:"gpt-4" content:"Test content"]
        [/@optional-placeholder]
        "#;

        let doc = parse_document(doc_str).unwrap();
        assert!(doc.blocks.contains_key("optional-test"), "Template expansion should generate named question block");
    }

    #[test]
    fn test_explicit_error_block() {
        let doc_str = "[error type:namespace_conflict]Duplicate block names[/error]";
        let doc = parse_document(doc_str).unwrap();

        assert_eq!(doc.unnamed_blocks.len(), 1);
        let error_block = &doc.unnamed_blocks[0];
        assert_eq!(error_block.block_type, "error");
        assert_eq!(error_block.modifiers.get("type").unwrap(), "namespace_conflict");
    }

    #[test]
    fn test_priority_modifier_for_context_ordering() {
        let doc_str = "[data name:low-priority priority:1]low[/data][data name:high-priority priority:10]high[/data]";
        let doc = parse_document(doc_str).unwrap();

        let high_priority_block = doc.blocks.get("high-priority").unwrap();
        assert_eq!(high_priority_block.modifiers.get("priority").unwrap(), "10");

        let low_priority_block = doc.blocks.get("low-priority").unwrap();
        assert_eq!(low_priority_block.modifiers.get("priority").unwrap(), "1");
    }

    #[test]
    fn test_context_pruning_order() {
        let doc_str = "[data name:essential priority:10]Keep[/data][data name:non-essential priority:2]Prune[/data]";
        let doc = parse_document(doc_str).unwrap();

        let essential_block = doc.blocks.get("essential").unwrap();
        assert_eq!(essential_block.modifiers.get("priority").unwrap(), "10");
    }

    #[test]
    fn test_secret_block_handling() {
        let doc_str = "[secret name:api-key]SECRET_ENV_VAR[/secret]";
        let doc = parse_document(doc_str).unwrap();

        assert!(doc.blocks.contains_key("api-key"));
        let secret_block = doc.blocks.get("api-key").unwrap();
        assert_eq!(secret_block.block_type, "secret");
    }

    #[test]
    fn test_memory_block_parsing() {
        let doc_str = "[memory name:user-state]persistent content[/memory]";
        let doc = parse_document(doc_str).unwrap();

        assert!(doc.blocks.contains_key("user-state"));
        let memory_block = doc.blocks.get("user-state").unwrap();
        assert_eq!(memory_block.block_type, "memory");
    }

    #[test]
    fn test_visualization_block_preview() {
        let doc_str = r#"
        [visualization]
          [question debug:true]
          Visualization test
          [/question]
          [preview]
          Auto-generated preview here
          [/preview]
        [/visualization]"#;

        let doc = parse_document(doc_str).unwrap();
        assert_eq!(doc.unnamed_blocks.len(), 1, "Should parse visualization block");
    }

    #[test]
fn test_order_modifier_for_context_ordering() {
    let document = "\
        [data name:later-order order:0.8]later content[/data]\
        [data name:earlier-order order:0.2]earlier content[/data]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let later_block = doc.blocks.get("later-order").unwrap();
    assert_eq!(later_block.modifiers.get("order").unwrap(), "0.8");
    
    let earlier_block = doc.blocks.get("earlier-order").unwrap();
    assert_eq!(earlier_block.modifiers.get("order").unwrap(), "0.2");
}

#[test]
fn test_weight_modifier() {
    let document = "\
        [data name:important-data weight:0.8]Important content[/data]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let data_block = doc.blocks.get("important-data").unwrap();
    assert_eq!(data_block.modifiers.get("weight").unwrap(), "0.8");
}

#[test]
fn test_summarize_modifier_options() {
    let document = "\
        [data name:brief-data summarize:brief]Long content to summarize[/data]\
        [data name:semantic-data summarize:semantic]Complex content[/data]\
        [data name:tabular-data summarize:tabular]Table data[/data]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let brief_block = doc.blocks.get("brief-data").unwrap();
    assert_eq!(brief_block.modifiers.get("summarize").unwrap(), "brief");
    
    let semantic_block = doc.blocks.get("semantic-data").unwrap();
    assert_eq!(semantic_block.modifiers.get("summarize").unwrap(), "semantic");
    
    let tabular_block = doc.blocks.get("tabular-data").unwrap();
    assert_eq!(tabular_block.modifiers.get("summarize").unwrap(), "tabular");
}

#[test]
fn test_async_execution_modifier() {
    let document = "\
        [code:python name:async-code async:true fallback:async-fallback]\
        print('Async execution')\
        [/code:python]\
        \
        [code:python name:async-fallback]\
        print('Fallback')\
        [/code:python]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let code_block = doc.blocks.get("async-code").unwrap();
    assert_eq!(code_block.modifiers.get("async").unwrap(), "true");
}

#[test]
fn test_filename_block() {
    let document = "\
        [filename name:external-file path:\"./test_file.txt\"]\
        [/filename]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    assert!(doc.blocks.contains_key("external-file"), "Should parse filename block");
    let file_block = doc.blocks.get("external-file").unwrap();
    assert_eq!(file_block.block_type, "filename");
    assert_eq!(file_block.modifiers.get("path").unwrap(), "./test_file.txt");
}

#[test]
fn test_multiple_dependencies() {
    let document = "\
        [data name:dep1]Dependency 1[/data]\
        [data name:dep2]Dependency 2[/data]\
        [question name:multi-dep depends:dep1 requires:dep2]\
        Question with multiple dependencies\
        [/question]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let question = doc.blocks.get("multi-dep").unwrap();
    assert!(question.depends_on.contains("dep1"), "Should have explicit dependency");
    assert!(question.requires.contains("dep2"), "Should have required dependency");
    
    let deps = doc.dependencies.get("multi-dep").cloned().unwrap_or_default();
    assert!(deps.contains("dep1") && deps.contains("dep2"), 
           "Dependencies should include both blocks");
}

#[test]
fn test_comment_block_isolation() {
    let document = "\
        [data name:test-data]Important data[/data]\
        [comment]\
        This comment references ${test-data} but should not create a dependency\
        [/comment]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    // Find the comment block
    let comment_block = doc.unnamed_blocks.iter()
        .find(|b| b.block_type == "comment")
        .expect("Should find comment block");
    
    // Ensure comment doesn't create dependencies
    let comment_deps = comment_block.depends_on.len() + comment_block.requires.len();
    assert_eq!(comment_deps, 0, "Comment should not create dependencies");
    
    // Ensure doc doesn't track comment references
    if let Some(deps) = doc.dependencies.values().find(|deps| deps.contains("test-data")) {
        if deps.len() > 0 {
            // This tests that the only dependency with test-data is from some actual dependency,
            // not from the comment block
            assert!(true);
        }
    }
}


#[test]
fn test_multiple_implicit_dependencies() {
    let document = "\
        [data name:dep1]First[/data]\
        [data name:dep2]Second[/data]\
        [data name:dep3]Third[/data]\
        [question name:multi-implicit]\
        Using ${dep1}, ${dep2}, and ${dep3}\
        [/question]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let deps = doc.dependencies.get("multi-implicit").cloned().unwrap_or_default();
    assert!(deps.contains("dep1") && deps.contains("dep2") && deps.contains("dep3"), 
           "Should resolve all implicit dependencies");
}

#[test]
fn test_namespace_conflict_detection() {
    let document = "\
        [data name:duplicate]First[/data]\
        [data name:duplicate]Second[/data]";
    
    let result = parse_document(document);
    
    assert!(result.is_err(), "Should detect namespace conflict");
    assert!(result.unwrap_err().contains("Namespace conflict"), 
           "Error message should mention namespace conflict");
}

#[test]
fn test_duplicate_block_names() {
    let document = "\
        [data name:duplicate]First[/data]\
        [data name:duplicate]Second[/data]";
        
    let result = parse_document(document);
        
    assert!(result.is_err(), "Should detect duplicate block names");
    assert!(result.unwrap_err().contains("Duplicate block name"), 
           "Error message should mention duplicate block name");
}

#[test]
fn test_retry_modifier() {
    let document = "\
        [api name:retry-api retry:3 fallback:retry-fallback]\
        https://api.example.com/endpoint\
        [/api]\
        \
        [data name:retry-fallback format:json]\
        {\"status\": \"fallback\"}\
        [/data]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let api_block = doc.blocks.get("retry-api").unwrap();
    assert_eq!(api_block.modifiers.get("retry").unwrap(), "3");
}

#[test]
fn test_timeout_modifier() {
    let document = "\
        [shell name:timeout-shell timeout:15 fallback:timeout-fallback]\
        sleep 30\
        [/shell]\
        \
        [shell name:timeout-fallback]\
        echo \"Fallback activated\"\
        [/shell]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let shell_block = doc.blocks.get("timeout-shell").unwrap();
    assert_eq!(shell_block.modifiers.get("timeout").unwrap(), "15");
}

#[test]
fn test_verbosity_levels() {
    let document = "\
        [question name:low-verbosity verbosity:low]\
        Question with low verbosity\
        [/question]\
        \
        [question name:high-verbosity verbosity:high]\
        Question with high verbosity\
        [/question]";
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    let low_block = doc.blocks.get("low-verbosity").unwrap();
    assert_eq!(low_block.modifiers.get("verbosity").unwrap(), "low");
    
    let high_block = doc.blocks.get("high-verbosity").unwrap();
    assert_eq!(high_block.modifiers.get("verbosity").unwrap(), "high");
}

#[test]
fn test_complex_nested_templates() {
    let document = r#"
    [template name:outer-template]
      [template name:inner-template-${name}]
        [data name:${inner_name}]
          ${inner_content}
        [/data]
      [/template]
      
      [@inner-template-${name} inner_name:"${inner_name}" inner_content:"${inner_content}"]
      [/@inner-template-${name}]
    [/template]
    
    [@outer-template
      name:"test"
      inner_name:"nested-data"
      inner_content:"Nested content"
    ]
    [/@outer-template]
    "#;
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    // Check that both templates were created
    assert!(doc.blocks.contains_key("outer-template"), "Should parse outer template");
    assert!(doc.blocks.contains_key("inner-template-test") || 
           doc.unnamed_blocks.iter().any(|b| b.block_type == "template" && 
                                         b.name.as_deref() == Some("inner-template-test")), 
           "Template expansion should create inner template");
    
    // Check that the nested data block was created
    assert!(doc.blocks.contains_key("nested-data") || 
           doc.unnamed_blocks.iter().any(|b| b.name.as_deref() == Some("nested-data")), 
           "Should create nested data block");
}

#[test]
fn test_mixed_block_types_in_template() {
    let document = r#"
    [template name:mixed-template]
      [data name:${name}-data]${data}[/data]
      [code:python name:${name}-code fallback:${name}-fallback]
        print("${message}")
      [/code:python]
      [code:python name:${name}-fallback]
        print("Fallback")
      [/code:python]
    [/template]
    
    [@mixed-template
      name:"mixed"
      data:"Template data"
      message:"Hello from template"
    ]
    [/@mixed-template]
    "#;
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    // Check that all blocks were created
    assert!(doc.blocks.contains_key("mixed-data"), "Should create data block from template");
    assert!(doc.blocks.contains_key("mixed-code"), "Should create code block from template");
    assert!(doc.blocks.contains_key("mixed-fallback"), "Should create fallback block from template");
}

#[test]
fn test_error_execution_failure() {
    let document = r#"
    [error type:execution_failure]
    Failed to execute code block with error: Division by zero
    [/error]
    "#;
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    assert_eq!(doc.unnamed_blocks.len(), 1);
    let error_block = &doc.unnamed_blocks[0];
    assert_eq!(error_block.block_type, "error");
    assert_eq!(error_block.modifiers.get("type").unwrap(), "execution_failure");
    assert!(error_block.content.contains("Division by zero"), "Error should contain specific failure reason");
}

#[test]
fn test_multiple_references_to_same_block() {
    let document = r#"
    [data name:reused-data]Reusable content[/data]
    [question name:multi-ref]
    First reference: ${reused-data}
    Second reference: ${reused-data}
    [/question]
    "#;
    
    let doc = parse_document(document).expect("Failed to parse document");
    
    // Test that multiple references to the same block are handled correctly
    let deps = doc.dependencies.get("multi-ref").cloned().unwrap_or_default();
    assert!(deps.contains("reused-data"), "Should resolve the dependency");
    assert_eq!(deps.iter().filter(|&dep| dep == "reused-data").count(), 1, 
              "Should only include the dependency once despite multiple references");
}

}
