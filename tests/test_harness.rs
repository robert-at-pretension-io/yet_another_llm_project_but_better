#[cfg(test)]
mod tests {
    // Include the main module from src/main.rs using the #[path] attribute.
    #[path = "../src/main.rs"]
    mod mainmod;

    use std::collections::HashSet;
    use mainmod::*;

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
}
