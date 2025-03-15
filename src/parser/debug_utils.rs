//! Debug utilities for the parser module

/// Log a debug message if the LLM_DEBUG environment variable is set
pub fn parser_debug(message: &str) {
    if std::env::var("LLM_DEBUG").is_ok() {
        println!("PARSER DEBUG: {}", message);
    }
}

/// Log the content of a document being parsed
pub fn log_document_content(content: &str) {
    if std::env::var("LLM_DEBUG").is_ok() {
        let preview = if content.len() > 200 {
            format!("{}... (length: {})", &content[..200], content.len())
        } else {
            content.to_string()
        };
        
        println!("PARSER DEBUG: Document content: {}", preview);
        
        // Log the first few lines
        let lines: Vec<&str> = content.lines().take(5).collect();
        println!("PARSER DEBUG: First {} lines:", lines.len());
        for (i, line) in lines.iter().enumerate() {
            println!("PARSER DEBUG: Line {}: {}", i+1, line);
        }
    }
}
