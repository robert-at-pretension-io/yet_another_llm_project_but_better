#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use yet_another_llm_project_but_better::*;

    #[test]
    fn test_parse_document_success() {
        let document = "[question name:sample]\nWhat is the answer?\n[/question]";
        let doc = parse_document(document).expect("Failed to parse document");
        assert!(doc.blocks.contains_key("sample"), "Document should contain a block named 'sample'");
    }

    #[test]
    fn test_resolve_dependencies() {
        let document = "[question name:sample depends:other]\nWhat?\n[/question][question name:other]\nOther question?\n[/question]";
        let doc = parse_document(document).expect("Failed to parse document");
        let deps: HashSet<String> = doc.dependencies.get("sample").cloned().unwrap_or_default();
        assert!(deps.contains("other"), "Dependencies should contain 'other'");
    }

    #[test]
    fn test_process_questions_no_error() {
        // Create a simple document with a question block.
        let document = "[question name:test_question model:default debug:true]\nCompute X\n[/question]";
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
              [code:python name:inner-code]\
              print(\"Inside section\")\
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
            [api name:test-api method:GET]\
            https://api.example.com/test\
            [/api]";
        
        let mut doc = parse_document(document).expect("Failed to parse document");
        let result = doc.execute_block("test-api").expect("API block should execute");
        
        assert!(result.contains("AI Response for"), "API block should return a response");
    }

    #[test]
    fn test_debug_mode() {
        let document = "\
            [question name:debug-question model:default debug:true]\
            Debug this question\
            [/question]";
        
        let mut doc = parse_document(document).expect("Failed to parse document");
        let result = doc.execute_block("debug-question").expect("Debug mode should execute");
        
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
        let document = "\
            [question name:already-answered]\
            What is 2+2?\
            [/question]\
            \
            [response name:already-answered-response depends:already-answered]\
            The answer is 4.\
            [/response]";
        
        let doc = parse_document(document).expect("Failed to parse document");
        
        let question = doc.blocks.get("already-answered").unwrap();
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
}
