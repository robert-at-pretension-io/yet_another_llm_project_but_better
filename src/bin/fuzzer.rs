use std::fs::File;
use std::io::Write;


use yet_another_llm_project_but_better::parser::parse_document;

/// A simple fuzzer for the meta language grammar
pub fn run_fuzzer(iterations: usize, max_mutations: usize, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting fuzzer with {} iterations", iterations);
    
    // Base templates for valid documents
    let templates = vec![
        r#"[data name:test]
content
[/data]"#,
        
        r#"[code:python name:test]
print("Hello")
[/code:python]"#,
        
        r#"[question model:gpt-4]
What is recursion?
[/question]"#,
        
        r#"[template name:test]
[code:python name:${name}]
print("${value}")
[/code:python]
[/template]"#,
    ];
    
    // Mutations to apply
    let mutations: Vec<Box<dyn Fn(&str) -> String>> = vec![
        // Replace [ with {
        Box::new(|s: &str| s.replace("[", "{")),
        
        // Replace ] with }
        Box::new(|s: &str| s.replace("]", "}")),
        
        // Remove name attribute
        Box::new(|s: &str| s.replace(" name:test", "")),
        
        // Add random attributes
        Box::new(|s: &str| {
            let attrs = vec![
                " debug:true",
                " cache_result:true",
                " async:true",
                " timeout:10",
                " fallback:test-fallback",
            ];
            let attr = attrs[rand::random::<usize>() % attrs.len()];
            s.replace("]", &format!("{attr}]"))
        }),
        
        // Insert random characters
        Box::new(|s: &str| {
            let pos = rand::random::<usize>() % s.len();
            let chars = "!@#$%^&*()_+";
            let random_char = chars.chars().nth(rand::random::<usize>() % chars.len()).unwrap();
            let mut result = s.to_string();
            result.insert(pos, random_char);
            result
        }),
        
        // Remove closing tag
        Box::new(|s: &str| {
            let mut lines: Vec<&str> = s.lines().collect();
            if lines.len() > 2 {
                lines.pop();
            }
            lines.join("\n")
        }),
        
        // Add invalid block type
        Box::new(|s: &str| s.replace("[data", "[invalid-type")),
        
        // Replace variable reference syntax
        Box::new(|s: &str| s.replace("${", "<<").replace("}", ">>")),
    ];
    
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut interesting_count = 0;
    
    for i in 0..iterations {
        if i % 100 == 0 {
            println!("Fuzzing iteration {}/{}...", i, iterations);
        }
        
        // Select a random template
        let template = templates[rand::random::<usize>() % templates.len()];
        
        // Apply random mutations
        let mut document = template.to_string();
        let num_mutations = rand::random::<usize>() % max_mutations + 1;
        
        for _ in 0..num_mutations {
            let mutation = &mutations[rand::random::<usize>() % mutations.len()];
            document = mutation(&document);
        }
        
        // Try to parse the mutated document
        match parse_document(&document) {
            Ok(blocks) => {
                success_count += 1;
                
                // Check if the document has interesting properties
                if blocks.len() > 0 && blocks.iter().any(|b| b.name.is_none()) {
                    // This might be interesting - a valid parse with unnamed blocks
                    interesting_count += 1;
                    save_interesting_case(&document, output_dir, i, "valid-unnamed")?;
                }
            },
            Err(e) => {
                failure_count += 1;
                
                // Check if the error contains certain patterns that might be interesting
                let error_str = format!("{:?}", e);
                if error_str.contains("stack overflow") || 
                   error_str.contains("unexpected end of input") {
                    interesting_count += 1;
                    save_interesting_case(&document, output_dir, i, "interesting-error")?;
                }
            }
        }
    }
    
    println!("\nFuzzing Results:");
    println!("Total iterations: {}", iterations);
    println!("Successful parses: {} ({:.2}%)", success_count, (success_count as f64 / iterations as f64) * 100.0);
    println!("Failed parses: {} ({:.2}%)", failure_count, (failure_count as f64 / iterations as f64) * 100.0);
    println!("Interesting cases: {}", interesting_count);
    println!("Saved interesting cases to: {}", output_dir);
    
    Ok(())
}

fn save_interesting_case(document: &str, output_dir: &str, iteration: usize, category: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure directory exists
    std::fs::create_dir_all(output_dir)?;
    
    // Save the document
    let filename = format!("{}/fuzz-{}-{}.md", output_dir, category, iteration);
    let mut file = File::create(&filename)?;
    writeln!(file, "# Fuzz Test Case: {}", category)?;
    writeln!(file, "# Iteration: {}", iteration)?;
    writeln!(file, "\n```markdown")?;
    write!(file, "{}", document)?;
    writeln!(file, "```")?;
    
    Ok(())
}

// Entry point for the fuzzer
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    let iterations = if args.len() > 1 {
        args[1].parse().unwrap_or(1000)
    } else {
        1000
    };
    
    let max_mutations = if args.len() > 2 {
        args[2].parse().unwrap_or(3)
    } else {
        3
    };
    
    let output_dir = if args.len() > 3 {
        args[3].to_string()
    } else {
        "fuzz_results".to_string()
    };
    
    run_fuzzer(iterations, max_mutations, &output_dir)
}