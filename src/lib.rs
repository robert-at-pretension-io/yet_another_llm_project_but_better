use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::sync::mpsc::channel;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::env;

fn extract_references(content: &str) -> HashSet<String> {
    println!("Debug: Extracting references from content: {}", content);
    let mut references = HashSet::new();
    let mut start = 0;

    while let Some(start_index) = content[start..].find("${") {
        start += start_index + 2;
        if let Some(end_index) = content[start..].find('}') {
            println!("Debug: Found reference: {}", &content[start..start + end_index]);
            let reference = &content[start..start + end_index];
            references.insert(reference.to_string());
            start += end_index + 1;
        } else {
            break;
        }
    }

    references
}
use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while},
    sequence::{delimited, preceded},
    multi::many0,
    character::complete::{alphanumeric1, alpha1, space0, space1, char},
    branch::alt,
    combinator::opt,
    Parser,
};

// Expanded Block structure to match the specification
#[derive(Debug, Clone)]
pub struct Block {
    pub block_type: String,
    pub name: Option<String>,
    pub modifiers: HashMap<String, String>,
    pub content: String,
    pub execution_result: Option<String>,
    pub depends_on: HashSet<String>,
    pub requires: HashSet<String>,
    pub answered: bool,  // Track if a question has been answered
}

// Document structure to manage blocks
#[derive(Debug)]
pub struct Document {
    pub blocks: HashMap<String, Block>,
    pub unnamed_blocks: Vec<Block>,
    pub dependencies: HashMap<String, HashSet<String>>,
}

// AI Service stub for integration with language models
struct AIService {
    model: String,
    api_key: Option<String>,
    api_endpoint: String,
    temperature: f32,
    max_tokens: usize,
}

impl AIService {
    fn new(model: &str) -> Self {
        AIService {
            model: model.to_string(),
            api_key: env::var("AI_API_KEY").ok(),
            api_endpoint: "https://api.example.com/v1/completions".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
        }
    }
    
    fn configure(&mut self, options: HashMap<String, String>) {
        if let Some(model) = options.get("model") {
            self.model = model.clone();
        }
        
        if let Some(temp) = options.get("temperature") {
            if let Ok(t) = temp.parse::<f32>() {
                self.temperature = t;
            }
        }
        
        if let Some(tokens) = options.get("max_tokens") {
            if let Ok(t) = tokens.parse::<usize>() {
                self.max_tokens = t;
            }
        }
        
        if let Some(endpoint) = options.get("api_endpoint") {
            self.api_endpoint = endpoint.clone();
        }
    }
    
    // This is a stub that you can replace with actual API calls
    fn generate_completion(&self, prompt: &str) -> Result<String, String> {
        println!("[AI Service] Would call {} with prompt:", self.model);
        println!("Temperature: {}, Max Tokens: {}", self.temperature, self.max_tokens);
        println!("Prompt:\n{}", prompt);
        
        // For now, return a placeholder response
        Ok(format!("AI Response for: {}", prompt.chars().take(50).collect::<String>()))
    }
}

impl Document {
    fn new() -> Self {
        Document {
            blocks: HashMap::new(),
            unnamed_blocks: Vec::new(),
            dependencies: HashMap::new(),
        }
    }
    
    // Method to mark a question as answered
    fn mark_question_answered(&mut self, question_name: &str) -> Result<(), String> {
        if let Some(block) = self.blocks.get_mut(question_name) {
            if block.block_type == "question" {
                block.answered = true;
                return Ok(());
            }
            return Err(format!("Block '{}' is not a question block", question_name));
        }
        Err(format!("Question block '{}' not found", question_name))
    }
    
    // Method to check if a question has been answered
    fn is_question_answered(&self, question_name: &str) -> bool {
        if let Some(block) = self.blocks.get(question_name) {
            return block.answered;
        }
        false
    }
    
    // Find an existing response block for a question
    fn find_response_for_question(&self, question_name: &str) -> Option<&Block> {
        self.blocks.values().find(|block| {
            block.block_type == "response" && 
            block.depends_on.contains(question_name)
        })
    }

    fn add_block(&mut self, block: Block) -> Result<(), String> {
        // Handle named blocks
        if let Some(name) = &block.name {
            if self.blocks.contains_key(name) {
                println!("Debug: Namespace conflict detected for block '{}'", name);
                return Err(format!("Namespace conflict: Block named '{}' already exists", name));
            }
            // Add to named blocks
            println!("Debug: Adding named block '{}'", name);
            self.blocks.insert(name.clone(), block);
        } else {
            // Add to unnamed blocks
            println!("Debug: Adding unnamed block");
            self.unnamed_blocks.push(block);
        }
        Ok(())
    }

    fn resolve_dependencies(&mut self) -> Result<(), String> {
        // Clear existing dependencies
        self.dependencies.clear();
        
        // Collect all explicit dependencies
        for (name, block) in &self.blocks {
            let mut deps = HashSet::new();
            deps.extend(block.depends_on.clone());
            deps.extend(block.requires.clone());
            
            // Add implicit dependencies from ${block_name} references
            let references = extract_references(&block.content);
            deps.extend(references);
            
            self.dependencies.insert(name.clone(), deps);
        }
        
        // Check for circular dependencies
        self.check_circular_dependencies()
    }
    
    fn check_circular_dependencies(&self) -> Result<(), String> {
        for name in self.dependencies.keys() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            
            if self.is_circular(name, &mut visited, &mut path) {
                return Err(format!("Circular dependency detected: {}", path.join(" -> ")));
            }
        }
        Ok(())
    }
    
    fn is_circular(&self, name: &str, visited: &mut HashSet<String>, path: &mut Vec<String>) -> bool {
        if path.contains(&name.to_string()) {
            path.push(name.to_string());
            return true;
        }
        
        if visited.contains(name) {
            return false;
        }
        
        visited.insert(name.to_string());
        path.push(name.to_string());
        
        if let Some(deps) = self.dependencies.get(name) {
            for dep in deps {
                if self.is_circular(dep, visited, path) {
                    return true;
                }
            }
        }
        
        path.pop();
        false
    }
    
    pub fn execute_block(&mut self, name: &str) -> Result<String, String> {
        println!("Debug: Executing block '{}'", name);
        println!("Debug: Attempting to retrieve block '{}'", name);
        println!("Debug: Attempting to retrieve block '{}'", name);
        let block = self.blocks.get(name).ok_or(format!("Block '{}' not found", name))?.clone();
        println!("Debug: Retrieved block '{}'", name);
        println!("Debug: Retrieved block '{}'", name);
        
        // Check if we have a cached execution result
        if let Some(result) = &block.execution_result {
            println!("Debug: Using cached result for block '{}'", name);
            return Ok(result.clone());
        }
        
        // Execute dependent blocks first
        let deps: Vec<String> = block.depends_on.iter().cloned().collect();
        println!("Debug: Block '{}' has dependencies: {:?}", name, deps);
        for dep in deps {
            println!("Debug: Executing dependency '{}'", dep);
            self.execute_block(&dep)?;
        }
        
        // Execute the block based on its type
        println!("Debug: Executing block type '{}'", block.block_type);
        println!("Debug: Executing block type '{}'", block.block_type);
        let result = match block.block_type.as_str() {
            "code" => self.execute_code_block(&block),
            "shell" => self.execute_shell_block(&block),
            "api" => self.execute_api_block(&block),
            "question" => self.build_context_for_question(&block),
            _ => Ok(block.content.clone()),
        }?;
        
        // Update the block with the execution result
        if let Some(block) = self.blocks.get_mut(name) {
            block.execution_result = Some(result.clone());
        }
        
        println!("Debug: Execution result for block '{}': {}", name, result);
        println!("Debug: Finished executing block '{}'", name);
        println!("Debug: Finished executing block '{}'", name);
        Ok(result)
    }
    
    fn execute_code_block(&self, block: &Block) -> Result<String, String> {
        // Get language from block_type (e.g., "code:python" -> "python")
        let lang = block.block_type.split(':').nth(1).unwrap_or("unknown");
        
        match lang {
            "python" => {
                let output = Command::new("python")
                    .arg("-c")
                    .arg(&block.content)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .map_err(|e| format!("Failed to execute Python code: {}", e))?;
                
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    // Check if there's a fallback
                    if let Some(fallback) = block.modifiers.get("fallback") {
                        if let Some(fallback_block) = self.blocks.get(fallback) {
                            return self.execute_code_block(fallback_block);
                        }
                    }
                    Err(String::from_utf8_lossy(&output.stderr).to_string())
                }
            },
            _ => Err(format!("Unsupported language: {}", lang)),
        }
    }
    
    fn execute_shell_block(&self, block: &Block) -> Result<String, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&block.content)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute shell command: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            if let Some(fallback) = block.modifiers.get("fallback") {
                if let Some(fallback_block) = self.blocks.get(fallback) {
                    return self.execute_shell_block(fallback_block);
                }
            }
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
    
    fn execute_api_block(&self, block: &Block) -> Result<String, String> {
        let url = block.content.trim();
        let default_method = "GET".to_string();
        let method = block.modifiers.get("method").unwrap_or(&default_method).as_str();
        
        let output = Command::new("curl")
            .arg("-X")
            .arg(method)
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute API request: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            if let Some(fallback) = block.modifiers.get("fallback") {
                if let Some(fallback_block) = self.blocks.get(fallback) {
                    return self.execute_api_block(fallback_block);
                }
            }
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
    
    fn build_context_for_question(&self, block: &Block) -> Result<String, String> {
        let mut context = String::new();
        context.push_str(&block.content);
        context.push_str("\n\n");
        
        for dep in &block.depends_on {
            if let Some(dep_block) = self.blocks.get(dep) {
                if let Some(result) = &dep_block.execution_result {
                    context.push_str(&format!("From {}: {}\n\n", dep, result));
                }
            }
        }
        
        for req in &block.requires {
            if let Some(req_block) = self.blocks.get(req) {
                context.push_str(&format!("Required {}: {}\n\n", req, req_block.content));
            }
        }
        
        let mut result = context.clone();
        let references = extract_references(&context);
        
        for ref_name in references {
            if let Some(ref_block) = self.blocks.get(&ref_name) {
                let placeholder = format!("${{{}}}", ref_name);
                let value = ref_block.execution_result.as_ref().unwrap_or(&ref_block.content);
                result = result.replace(&placeholder, value);
            }
        }
        
        let model = block.modifiers.get("model").unwrap_or(&"default".to_string()).clone();
        
        let mut ai_service = AIService::new(&model);
        ai_service.configure(block.modifiers.clone());
        
        if block.modifiers.get("debug").unwrap_or(&"false".to_string()) == "true" {
            Ok(format!("DEBUG CONTEXT:\n{}", result))
        } else {
            ai_service.generate_completion(&result)
        }
    }
    
    fn process_templates(&mut self) -> Result<(), String> {
        let mut template_invocations = Vec::new();
        let mut i = 0;
        
        while i < self.unnamed_blocks.len() {
            let block = &self.unnamed_blocks[i];
            
            if block.block_type.starts_with('@') {
                let template_name = block.block_type.trim_start_matches('@');
                
                let mut closing_idx = None;
                for j in i+1..self.unnamed_blocks.len() {
                    if self.unnamed_blocks[j].block_type == format!("/@{}", template_name) {
                        closing_idx = Some(j);
                        break;
                    }
                }
                
                if let Some(end_idx) = closing_idx {
                    template_invocations.push((i, end_idx, template_name.to_string()));
                    i = end_idx + 1;
                    continue;
                }
            }
            
            i += 1;
        }
        
        template_invocations.sort_by(|a, b| b.0.cmp(&a.0));
        
        for (start_idx, end_idx, template_name) in template_invocations {
            let template = self.blocks.get(&template_name).ok_or(
                format!("Template '{}' not found", template_name)
            )?;
            
            let invocation = &self.unnamed_blocks[start_idx];
            let parameters = &invocation.modifiers;
            let mut expanded_content = template.content.clone();
            
            for (param_name, param_value) in parameters {
                let placeholder = format!("${{{}}}", param_name);
                expanded_content = expanded_content.replace(&placeholder, param_value);
            }
            
            let mut expanded_modifiers = template.modifiers.clone();
            for (key, value) in &invocation.modifiers {
                expanded_modifiers.insert(key.clone(), value.clone());
            }
            
            let (_, blocks) = many0(parse_block).parse(expanded_content.as_str())
                .map_err(|e| format!("Failed to parse template content: {:?}", e))?;
            
            self.unnamed_blocks.drain(start_idx..=end_idx);
            
            for (idx, block) in blocks.into_iter().enumerate() {
                self.unnamed_blocks.insert(start_idx + idx, block);
            }
        }
        
        Ok(())
    }
    
    fn expand_template(&self, template_name: &str, parameters: HashMap<String, String>) -> Result<Vec<Block>, String> {
        let template = self.blocks.get(template_name).ok_or(
            format!("Template '{}' not found", template_name)
        )?;
        
        let mut expanded_content = template.content.clone();
        
        for (param_name, param_value) in &parameters {
            let placeholder = format!("${{{}}}", param_name);
            expanded_content = expanded_content.replace(&placeholder, param_value);
        }
        
        let mut expanded_modifiers = template.modifiers.clone();
        for (key, value) in &parameters {
            if key.contains(":") {
                let parts: Vec<&str> = key.split(':').collect();
                if parts.len() == 2 {
                    expanded_modifiers.insert(parts[1].to_string(), value.clone());
                }
            }
        }
        
        let (_, blocks) = many0(parse_block).parse(expanded_content.as_str())
            .map_err(|e| format!("Failed to parse template content: {:?}", e))?;
        
        Ok(blocks)
    }
}


fn parse_modifier(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = alphanumeric1(input)?;
    let (input, _) = char(':')(input)?;
    
    let mut value_parser = alt((
        delimited(char('"'), take_while(|c| c != '"'), char('"')),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    ));
    
    let (input, value) = value_parser.parse(input)?;
    
    Ok((input, (key.to_string(), value.to_string())))
}

fn parse_block_header(input: &str) -> IResult<&str, (String, Option<String>, HashMap<String, String>)> {
    let (input, _) = tag("[")(input)?;
    
    let (input, block_type_start) = alt((alpha1, tag("@"))).parse(input)?;
    
    let (input, block_type_suffix) = opt(preceded(char(':'), alphanumeric1)).parse(input)?;
    
    let block_type = match block_type_suffix {
        Some(suffix) => format!("{}:{}", block_type_start, suffix),
        None => block_type_start.to_string(),
    };
    
    let (input, name_prefix) = opt(preceded(space1, tag("name:"))).parse(input)?;
    let (input, name) = match name_prefix {
        Some(_) => {
            let (input, name_value) = alt((
                delimited(char('"'), take_while(|c| c != '"'), char('"')),
                alphanumeric1
            )).parse(input)?;
            (input, Some(name_value.to_string()))
        },
        None => (input, None),
    };
    
    let (input, modifiers_str) = take_until("]")(input)?;
    
    let mut modifiers = HashMap::new();
    if !modifiers_str.is_empty() {
        let (_, modifiers_list) = many0(preceded(space1, parse_modifier)).parse(modifiers_str)?;
        for (key, value) in modifiers_list {
            modifiers.insert(key, value);
        }
    }
    
    let (input, _) = tag("]")(input)?;
    
    Ok((input, (block_type, name, modifiers)))
}

fn parse_block(input: &str) -> IResult<&str, Block> {
    let (input, _) = space0(input)?;
    let (input, (block_type, name, modifiers)) = parse_block_header(input)?;
    
    if block_type.starts_with("/@") {
        return Ok((input, Block {
            block_type: block_type,
            name: name,
            modifiers: modifiers,
            content: "".to_string(),
            execution_result: None,
            depends_on: HashSet::new(),
            requires: HashSet::new(),
            answered: false,
        }));
    }
    
    let end_tag = format!("[/{}]", block_type.split(':').next().unwrap_or(&block_type));
    let (input, content) = take_until(end_tag.as_str())(input)?;
    let (input, _) = tag(end_tag.as_str())(input)?;
    
    let mut depends_on = HashSet::new();
    let mut requires = HashSet::new();
    
    if let Some(deps) = modifiers.get("depends") {
        depends_on.insert(deps.clone());
    }
    
    if let Some(reqs) = modifiers.get("requires") {
        requires.insert(reqs.clone());
    }
    
    let answered = block_type == "response";
    
    Ok((input, Block {
        block_type,
        name,
        modifiers,
        content: content.trim().to_string(),
        execution_result: None,
        depends_on,
        requires,
        answered,
    }))
}

pub fn run() {
    // Get file path from command line argument or use default
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).unwrap_or(&"./document.meta".to_string()).clone();
    
    match fs::read_to_string(&path) {
        Ok(content) => {
            match parse_document(&content) {
                Ok(mut doc) => {
                    println!("Successfully parsed document with {} named blocks and {} unnamed blocks",
                          doc.blocks.len(), doc.unnamed_blocks.len());
                    
                    if let Err(e) = process_questions(&mut doc) {
                        println!("Error processing questions: {}", e);
                    }
                    
                    if let Err(e) = write_responses_to_file(&doc, &path) {
                        println!("Error writing responses: {}", e);
                    }
                },
                Err(e) => println!("Error parsing document: {}", e),
            }
        },
        Err(e) => println!("Error reading file: {}", e),
    }
    
    watch_file(&path);
}

fn write_responses_to_file(doc: &Document, path: &str) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read document: {}", e))?;
    
    let mut new_content = content.clone();
    
    for (_name, block) in &doc.blocks {
        if block.block_type == "response" {
            let question_name = block.depends_on.iter().next().cloned();
            
            if let Some(q_name) = question_name {
                if let Some(_question) = doc.blocks.get(&q_name) {
                    let question_tag = format!("[/question]");
                    
                    if let Some(pos) = new_content.find(&question_tag) {
                        let insert_pos = pos + question_tag.len();
                        
                        let response_text = format!(
                            "\n\n[response name:{}]\n{}\n[/response]",
                            block.name.clone().unwrap_or_default(),
                            block.content
                        );
                        
                        if !new_content.contains(&response_text) {
                            new_content.insert_str(insert_pos, &response_text);
                        }
                    }
                }
            }
        }
    }
    
    if new_content != content {
        fs::write(path, new_content)
            .map_err(|e| format!("Failed to write document: {}", e))?;
        println!("Updated document with new responses");
    }
    
    Ok(())
}

fn watch_file(path: &str) {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())
        .expect("Failed to create watcher");
    
    watcher.watch(std::path::Path::new(&path), RecursiveMode::NonRecursive)
        .expect("Failed to watch file");
    
    println!("Watching file: {}", path);
    
    loop {
        match rx.recv() {
            Ok(Ok(Event { kind: EventKind::Modify(_), .. })) => {
                println!("File changed, re-parsing...");
                std::thread::sleep(Duration::from_millis(100));
                
                match fs::read_to_string(path) {
                    Ok(content) => {
                        match parse_document(&content) {
                            Ok(mut doc) => {
                                println!("Successfully parsed document with {} named blocks and {} unnamed blocks",
                                      doc.blocks.len(), doc.unnamed_blocks.len());
                                
                                for name in doc.blocks.keys().cloned().collect::<Vec<_>>() {
                                    let block_type = doc.blocks.get(&name)
                                        .map(|b| b.block_type.clone())
                                        .unwrap_or_default();
                                    
                                    if block_type.starts_with("code:") || 
                                       block_type == "shell" || 
                                       block_type == "api" {
                                        match doc.execute_block(&name) {
                                            Ok(_) => println!("Block '{}' executed successfully", name),
                                            Err(e) => println!("Block '{}' execution failed: {}", name, e),
                                        }
                                    }
                                }
                                
                                if let Err(e) = process_questions(&mut doc) {
                                    println!("Error processing questions: {}", e);
                                } else {
                                    if let Err(e) = write_responses_to_file(&doc, path) {
                                        println!("Error writing responses: {}", e);
                                    }
                                }
                                
                                if let Err(e) = process_visualizations(&mut doc) {
                                    println!("Error processing visualizations: {}", e);
                                }
                                
                                println!("Document processing complete");
                            },
                            Err(e) => println!("Error parsing document: {}", e),
                        }
                    },
                    Err(e) => println!("Error reading file: {}", e),
                }
            },
            Err(e) => println!("Watch error: {:?}", e),
            _ => {}
        }
    }
}

pub fn process_questions(doc: &mut Document) -> Result<(), String> {
    let question_blocks: Vec<String> = doc.blocks.iter()
        .filter(|(_, block)| block.block_type == "question" && !block.answered)
        .map(|(name, _)| name.clone())
        .collect();
    
    if question_blocks.is_empty() {
        println!("No new questions to process");
        return Ok(());
    }
    
    println!("Found {} unanswered questions", question_blocks.len());
    
    for name in question_blocks {
        if let Some(response) = doc.find_response_for_question(&name) {
            println!("Question '{}' already has a response block: {}", 
                     name, response.name.clone().unwrap_or_default());
            
            doc.mark_question_answered(&name)?;
            continue;
        }
        
        println!("Processing question block: {}", name);
        
        match doc.execute_block(&name) {
            Ok(response) => {
                let response_block = Block {
                    block_type: "response".to_string(),
                    name: Some(format!("{}-response", name)),
                    modifiers: HashMap::new(),
                    content: response,
                    execution_result: None,
                    depends_on: {
                        let mut deps = HashSet::new();
                        deps.insert(name.clone());
                        deps
                    },
                    requires: HashSet::new(),
                    answered: false,
                };
                
                doc.add_block(response_block)?;
                doc.mark_question_answered(&name)?;
                
                println!("Generated response for question: {}", name);
            },
            Err(e) => {
                println!("Failed to process question {}: {}", name, e);
                
                let error_block = Block {
                    block_type: "error".to_string(),
                    name: Some(format!("{}-error", name)),
                    modifiers: {
                        let mut mods = HashMap::new();
                        mods.insert("type".to_string(), "execution_failure".to_string());
                        mods
                    },
                    content: format!("Failed to process question: {}", e),
                    execution_result: None,
                    depends_on: HashSet::new(),
                    requires: HashSet::new(),
                    answered: false,
                };
                
                doc.add_block(error_block)?;
            }
        }
    }
    
    Ok(())
}

fn process_visualizations(doc: &mut Document) -> Result<(), String> {
    let visualization_blocks: Vec<usize> = doc.unnamed_blocks.iter()
        .enumerate()
        .filter(|(_, block)| block.block_type == "visualization")
        .map(|(idx, _)| idx)
        .collect();
    
    for idx in visualization_blocks {
        if idx < doc.unnamed_blocks.len() {
            let block = &doc.unnamed_blocks[idx];
            println!("Processing visualization block at index {}", idx);
            println!("Visualization content: {}", block.content);
        }
    }
    
    Ok(())
}

pub fn parse_document(input: &str) -> Result<Document, String> {
    let mut doc = Document::new();
    let (_, blocks) = many0(parse_block).parse(input)
        .map_err(|e| format!("Parsing error: {:?}", e))?;
    
    for block in blocks {
        doc.add_block(block)?;
    }
    
    let mut questions_to_mark = Vec::new();
    
    for (_name, block) in &doc.blocks {
        if block.block_type == "response" {
            for dep in &block.depends_on {
                if let Some(question_block) = doc.blocks.get(dep) {
                    if question_block.block_type == "question" {
                        questions_to_mark.push(dep.clone());
                    }
                }
            }
        }
    }
    
    for question_name in questions_to_mark {
        if let Some(question) = doc.blocks.get_mut(&question_name) {
            question.answered = true;
            println!("Marking question '{}' as already answered", question_name);
        }
    }
    
    doc.resolve_dependencies()?;
    doc.process_templates()?;
    doc.resolve_dependencies()?;
    
    Ok(doc)
}
