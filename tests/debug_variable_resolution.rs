use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::time::Instant;

use yet_another_llm_project_but_better::executor::MetaLanguageExecutor;
use yet_another_llm_project_but_better::parser::parse_document;
use yet_another_llm_project_but_better::parser::Block;

/// This test focuses on understanding variable reference resolution
/// It creates a simple document with a variable and a reference to it,
/// then traces the parsing and resolution process in detail.
#[test]
fn test_basic_variable_resolution() {
    // Enable debug logging
    std::env::set_var("LLM_DEBUG", "1");
    
    println!("\n===== TESTING VARIABLE RESOLUTION =====");
    
    // Create a simple document with a data block and a reference to it
    let document = r#"
<meta:document xmlns:meta="https://example.com/meta-language">
    <meta:data name="my_variable">
This is my variable content
    </meta:data>
    
    <meta:code:python name="use_variable">
# This code references a variable
print("The variable content is: <meta:reference name='my_variable' />")
    </meta:code:python>
</meta:document>
"#;
    
    println!("\n--- Document Content ---");
    println!("{}", document);
    
    // STEP 1: Low-level XML parsing to see how references are represented
    println!("\n--- STEP 1: Raw XML Events ---");
    let mut reader = Reader::from_str(document);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut depth = 0;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                println!("{:indent$}Start: {:?}", "", e.name(), indent=depth*2);
                depth += 1;
            },
            Ok(Event::End(ref e)) => {
                depth -= 1;
                println!("{:indent$}End: {:?}", "", e.name(), indent=depth*2);
            },
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default();
                println!("{:indent$}Text: {:?}", "", text, indent=depth*2);
            },
            Ok(Event::Empty(ref e)) => {
                println!("{:indent$}Empty: {:?}", "", e.name(), indent=depth*2);
                
                // Check for reference tags specifically
                if e.name().as_ref() == b"meta:reference" {
                    println!("{:indent$}  Found reference tag!", "", indent=depth*2);
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            println!("{:indent$}  Attribute: {:?} = {:?}", "", 
                                     String::from_utf8_lossy(attr.key.as_ref()),
                                     String::from_utf8_lossy(&attr.value), 
                                     indent=depth*2);
                        }
                    }
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            },
            _ => {},
        }
        buf.clear();
    }
    
    // STEP 2: Document parsing with the actual parser
    println!("\n--- STEP 2: Document Parser Results ---");
    match parse_document(document) {
        Ok(blocks) => {
            println!("✅ Parsing succeeded! Found {} blocks", blocks.len());
            for (i, block) in blocks.iter().enumerate() {
                println!("\nBlock {}: type={}, name={:?}", 
                    i, block.block_type, block.name);
                println!("Content:");
                println!("```");
                println!("{}", block.content);
                println!("```");
                
                // Check if this block contains a reference
                if block.content.contains("<meta:reference") {
                    println!("⚠️ Block contains XML reference tag!");
                }
                // Check for special marker
                if block.content.contains("__META_REFERENCE__") {
                    println!("⚠️ Block contains special reference marker!");
                }
                // Check if it contains ${var} style reference
                if block.content.contains("${") {
                    println!("⚠️ Block contains shell-style reference!");
                }
            }
            
            // STEP 3: Test with executor
            println!("\n--- STEP 3: Variable Resolution with Executor ---");
            let mut executor = MetaLanguageExecutor::new();
            executor.current_document = document.to_string();
            
            // Register all blocks
            for block in &blocks {
                if let Some(name) = &block.name {
                    executor.blocks.insert(name.clone(), block.clone());
                    println!("Registered block: {}", name);
                    
                    // Store outputs for data blocks immediately
                    if block.block_type == "data" {
                        executor.outputs.insert(name.clone(), block.content.trim().to_string());
                        println!("Stored output for data block: {} = '{}'", name, block.content.trim());
                    }
                }
            }
            
            // STEP 4: Focus on the variable resolution process
            println!("\n--- STEP 4: Variable Reference Processing Details ---");
            if let Some(python_block) = blocks.iter().find(|b| b.block_type == "code:python") {
                println!("Testing variable references in block: {}", 
                         python_block.name.as_ref().unwrap_or(&"unnamed".to_string()));
                
                println!("\nOriginal content:");
                println!("```");
                println!("{}", python_block.content);
                println!("```");
                
                // Check current state of outputs
                println!("\nCurrent executor outputs:");
                for (k, v) in &executor.outputs {
                    println!("  {} = '{}'", k, v);
                }
                
                // Process variable references
                match executor.process_variable_references(&python_block.content) {
                    Ok(processed) => {
                        println!("\nProcessed content:");
                        println!("```");
                        println!("{}", processed);
                        println!("```");
                        
                        // Compare input/output if they differ
                        if python_block.content != processed {
                            println!("\n✅ Variable reference was processed!");
                            
                            // Find the exact position and contents of the replacement
                            let mut start_pos = None;
                            let mut end_pos = None;
                            
                            // Find common prefix
                            for (i, (a, b)) in python_block.content.chars().zip(processed.chars()).enumerate() {
                                if a != b {
                                    start_pos = Some(i);
                                    break;
                                }
                            }
                            
                            // Find common suffix (working backwards)
                            let orig_chars: Vec<char> = python_block.content.chars().collect();
                            let proc_chars: Vec<char> = processed.chars().collect();
                            
                            for i in 0..std::cmp::min(orig_chars.len(), proc_chars.len()) {
                                if orig_chars[orig_chars.len() - 1 - i] != proc_chars[proc_chars.len() - 1 - i] {
                                    end_pos = Some(std::cmp::min(orig_chars.len(), proc_chars.len()) - i);
                                    break;
                                }
                            }
                            
                            if let (Some(start), Some(end)) = (start_pos, end_pos) {
                                println!("\nReplacement details:");
                                println!("  Start position: {}", start);
                                println!("  End position: {}", end);
                                println!("  In original: '{}'", &python_block.content[start..std::cmp::min(end, python_block.content.len())]);
                                println!("  In processed: '{}'", &processed[start..std::cmp::min(end, processed.len())]);
                            }
                        } else {
                            println!("\n❌ Variable reference was NOT processed - content unchanged!");
                        }
                    },
                    Err(e) => {
                        println!("\n❌ Error processing references: {:?}", e);
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ Parsing failed: {:?}", e);
        }
    }
}
