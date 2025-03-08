// meta_lang_parser/src/lib.rs

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use regex::Regex;
use thiserror::Error;

/// Re-exports important types and traits for consumers of this library
pub mod prelude {
    pub use crate::{
        BlockType, BlockContent, Block, BlockId, DocumentParser, 
        BlockProcessor, BlockProcessorPlugin, ParserPlugin, ParserConfig,
        Error, ProcessorRegistry, BlockProcessorContext,
    };
}

/// Errors that can occur during document parsing and processing
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error at position {pos}: {message}")]
    ParseError { pos: usize, message: String },
    
    #[error("Namespace conflict: Block name '{0}' is defined more than once")]
    NamespaceConflict(String),
    
    #[error("Circular dependency detected between {0} and {1}")]
    CircularDependency(String, String),
    
    #[error("Required fallback block '{0}' not found for executable block '{1}'")]
    MissingFallback(String, String),
    
    #[error("Block execution error: {0}")]
    ExecutionError(String),
    
    #[error("Missing dependency: Block '{0}' requires '{1}' which was not found")]
    MissingDependency(String, String),
    
    #[error("Invalid block reference: '{0}'")]
    InvalidBlockReference(String),
}

/// The result type for operations that can fail
pub type Result<T> = std::result::Result<T, Error>;

/// A unique identifier for a block in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(Entity);

impl BlockId {
    /// Creates a new BlockId from a Bevy Entity
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
    
    /// Gets the underlying Bevy Entity
    pub fn entity(&self) -> Entity {
        self.0
    }
}

/// Represents the different types of blocks in the document
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Code(String),
    Question,
    Response,
    Data,
    Shell,
    Api,
    Template,
    Comment,
    Variable,
    Secret,
    Error,
    Visualization,
    Memory,
    Conditional,
    Debug,
    Filename,
    Section(String),
    /// User-defined extension block types
    Extension(String),
}

impl BlockType {
    /// Determines if this block type requires a fallback block
    pub fn requires_fallback(&self) -> bool {
        matches!(self, 
            BlockType::Code(_) | 
            BlockType::Shell | 
            BlockType::Api
        )
    }
    
    /// Converts a string representation to a BlockType
    pub fn from_str(block_type_str: &str) -> Option<Self> {
        if block_type_str.starts_with("code:") {
            let language = block_type_str.strip_prefix("code:").unwrap_or("").to_string();
            Some(BlockType::Code(language))
        } else if block_type_str.starts_with("section:") {
            let section_type = block_type_str.strip_prefix("section:").unwrap_or("").to_string();
            Some(BlockType::Section(section_type))
        } else {
            match block_type_str {
                "question" => Some(BlockType::Question),
                "response" => Some(BlockType::Response),
                "data" => Some(BlockType::Data),
                "shell" => Some(BlockType::Shell),
                "api" => Some(BlockType::Api),
                "template" => Some(BlockType::Template),
                "comment" => Some(BlockType::Comment),
                "variable" => Some(BlockType::Variable),
                "secret" => Some(BlockType::Secret),
                "error" => Some(BlockType::Error),
                "visualization" => Some(BlockType::Visualization),
                "memory" => Some(BlockType::Memory),
                "conditional" => Some(BlockType::Conditional),
                "debug" => Some(BlockType::Debug),
                "filename" => Some(BlockType::Filename),
                _ => Some(BlockType::Extension(block_type_str.to_string())),
            }
        }
    }
    
    /// Converts this BlockType to its string representation
    pub fn as_str(&self) -> String {
        match self {
            BlockType::Code(lang) => format!("code:{}", lang),
            BlockType::Section(section_type) => format!("section:{}", section_type),
            BlockType::Question => "question".to_string(),
            BlockType::Response => "response".to_string(),
            BlockType::Data => "data".to_string(),
            BlockType::Shell => "shell".to_string(),
            BlockType::Api => "api".to_string(),
            BlockType::Template => "template".to_string(),
            BlockType::Comment => "comment".to_string(),
            BlockType::Variable => "variable".to_string(),
            BlockType::Secret => "secret".to_string(),
            BlockType::Error => "error".to_string(),
            BlockType::Visualization => "visualization".to_string(),
            BlockType::Memory => "memory".to_string(),
            BlockType::Conditional => "conditional".to_string(),
            BlockType::Debug => "debug".to_string(),
            BlockType::Filename => "filename".to_string(),
            BlockType::Extension(name) => name.clone(),
        }
    }
}

/// The content of a block, with a strongly typed representation
#[derive(Debug, Clone)]
pub enum BlockContent {
    Text(String),
    Json(serde_json::Value),
    Code { language: String, source: String },
    Api { url: String, method: String, headers: HashMap<String, String> },
    Shell(String)
}

impl Default for BlockContent {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

/// The main component representing a block in the document
#[derive(Component, Debug)]
pub struct Block {
    pub block_type: BlockType,
    pub name: Option<String>,
    pub content: BlockContent,
    pub modifiers: HashMap<String, String>,
    pub depends_on: HashSet<String>,
    pub requires: HashSet<String>,
    pub fallback: Option<String>,
    pub executed: bool,
    pub output: Option<BlockContent>,
    pub error: Option<String>,
}

/// Events for communication between systems
#[derive(Event, Debug)]
pub struct BlockParsedEvent {
    pub block_id: BlockId,
    pub name: Option<String>,
}

#[derive(Event, Debug)]
pub struct BlockExecutedEvent {
    pub block_id: BlockId,
    pub output: Option<BlockContent>,
    pub error: Option<String>,
}

#[derive(Event, Debug)]
pub struct BlockExecutionRequestEvent {
    pub block_id: BlockId,
}

/// Resource that holds the map of named blocks
#[derive(Resource, Debug, Default)]
pub struct NamedBlocks(pub HashMap<String, BlockId>);

/// Resource that holds the execution queue
#[derive(Resource, Debug, Default)]
pub struct ExecutionState {
    pub queue: VecDeque<BlockId>,
    pub executed: HashSet<BlockId>,
    pub pending: HashSet<BlockId>,
    pub dependencies: HashMap<String, HashSet<String>>,
}

/// Resource that holds the parsing state
#[derive(Resource, Debug)]
pub struct ParsingState {
    pub document: String,
    pub current_position: usize,
    pub is_complete: bool,
    pub block_start_regex: Regex,
    pub block_end_regex: Regex,
    pub reference_regex: Regex,
}

impl Default for ParsingState {
    fn default() -> Self {
        Self {
            document: String::new(),
            current_position: 0,
            is_complete: false,
            block_start_regex: Regex::new(r"\[([\w:]+)(?:\s+([^\]]+))?\]").unwrap(),
            block_end_regex: Regex::new(r"\[/([\w:]+)\]").unwrap(),
            reference_regex: Regex::new(r"\$\{([a-zA-Z0-9_-]+)\}").unwrap(),
        }
    }
}

/// Configuration for the parser
#[derive(Resource,Clone)]
pub struct ParserConfig {
    pub debug_mode: bool,
    pub allow_circular_dependencies: bool,
    pub auto_execute: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            allow_circular_dependencies: false,
            auto_execute: true,
        }
    }
}

/// The central registry of block processors
#[derive(Resource, Default)]
pub struct ProcessorRegistry {
    processors: HashMap<String, Box<dyn BlockProcessor + Send + Sync>>,
}

impl ProcessorRegistry {
    /// Registers a block processor for a specific block type
    pub fn register<P: BlockProcessor + Send + Sync + 'static>(&mut self, processor: P) {
        let processor_type = processor.block_type();
        self.processors.insert(processor_type, Box::new(processor));
    }
    
    /// Gets a processor for a specific block type
    pub fn get(&self, block_type: &str) -> Option<&(dyn BlockProcessor + Send + Sync)> {
        self.processors.get(block_type).map(|p| p.as_ref())
    }
}

/// Context provided to block processors
pub struct BlockProcessorContext<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub named_blocks: &'a mut NamedBlocks,
    pub execution_state: &'a mut ExecutionState,
    pub world: &'a mut World,
}

/// Trait for block processors
pub trait BlockProcessor {
    /// Returns the block type this processor handles
    fn block_type(&self) -> String;
    
    /// Processes a parsed block
    fn process(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<Option<BlockContent>>;
    
    /// Called when the block is parsed
    fn on_parse(&self, _block_id: BlockId, _ctx: &mut BlockProcessorContext) -> Result<()> {
        Ok(())
    }
    
    /// Called when the processor is registered
    fn on_register(&self) {}
}

/// Plugin trait for block processors
pub trait BlockProcessorPlugin: Plugin {
    /// Returns the block processor this plugin provides
    fn processor(&self) -> Box<dyn BlockProcessor + Send + Sync>;
}

/// The main document parser
#[derive(Resource)]
pub struct DocumentParser {
    pub document: String,
}

impl DocumentParser {
    /// Creates a new document parser
    pub fn new(document: String) -> Self {
        Self { document }
    }
    
    /// Parses the document and adds the blocks to the world
    pub fn parse(&self, world: &mut World) -> Result<()> {
        let mut parsing_state = ParsingState {
            document: self.document.clone(),
            ..Default::default()
        };
        
        while !parsing_state.is_complete {
            self.parse_next_block(world, &mut parsing_state)?;
        }
        
        // Validate dependencies and fallbacks
        self.validate_document(world)?;
        
        Ok(())
    }
    
    /// Parses the next block in the document
    fn parse_next_block(&self, world: &mut World, state: &mut ParsingState) -> Result<()> {
        if state.is_complete {
            return Ok(());
        }
        
        let document = &state.document;
        let current_position = state.current_position;
        
        // Find the next block start
        if let Some(start_match) = state.block_start_regex.find_at(document, current_position) {
            let start_pos = start_match.start();
            let end_pos = start_match.end();
            
            // Extract block type and modifiers
            let captures = state.block_start_regex.captures_at(document, current_position).unwrap();
            let block_type_str = captures.get(1).unwrap().as_str();
            
            // Parse regular block
            if let Some(block_type) = BlockType::from_str(block_type_str) {
                // Parse modifiers and name
                let mut modifiers = HashMap::new();
                let mut name = None;
                let mut depends_on = HashSet::new();
                let mut requires = HashSet::new();
                let mut fallback = None;
                
                if let Some(modifier_str) = captures.get(2) {
                    for pair in modifier_str.as_str().split_whitespace() {
                        if let Some((key, value)) = pair.split_once(':') {
                            // Handle special modifiers
                            match key {
                                "name" => {
                                    let name_value = value.trim_matches(|c| c == '"' || c == '\'');
                                    name = Some(name_value.to_string());
                                },
                                "depends" => {
                                    let dep_value = value.trim_matches(|c| c == '"' || c == '\'');
                                    depends_on.insert(dep_value.to_string());
                                },
                                "requires" => {
                                    let req_value = value.trim_matches(|c| c == '"' || c == '\'');
                                    requires.insert(req_value.to_string());
                                },
                                "fallback" => {
                                    let fallback_value = value.trim_matches(|c| c == '"' || c == '\'');
                                    fallback = Some(fallback_value.to_string());
                                },
                                _ => {
                                    modifiers.insert(key.to_string(), value.to_string());
                                }
                            }
                        }
                    }
                }
                
                // Find the end of this block
                let end_tag = format!("[/{block_type_str}]");
                if let Some(end_pos_rel) = document[end_pos..].find(&end_tag) {
                    let content_end = end_pos + end_pos_rel;
                    let content = document[end_pos..content_end].trim().to_string();
                    
                    // Create block entity
                    let mut commands = Commands::new(world);
                    
                    // Create block content based on type
                    let block_content = match &block_type {
                        BlockType::Code(language) => {
                            BlockContent::Code { 
                                language: language.clone(), 
                                source: content.clone() 
                            }
                        },
                        BlockType::Data => {
                            if let Some(format) = modifiers.get("format") {
                                if format == "json" {
                                    if let Ok(json) = serde_json::from_str(&content) {
                                        BlockContent::Json(json)
                                    } else {
                                        BlockContent::Text(content.clone())
                                    }
                                } else {
                                    BlockContent::Text(content.clone())
                                }
                            } else {
                                BlockContent::Text(content.clone())
                            }
                        },
                        BlockType::Shell => BlockContent::Shell(content.clone()),
                        BlockType::Api => {
                            let method = modifiers.get("method")
                                .map(|s| s.clone())
                                .unwrap_or_else(|| "GET".to_string());
                            
                            BlockContent::Api { 
                                url: content.clone(), 
                                method, 
                                headers: HashMap::new() 
                            }
                        },
                        _ => BlockContent::Text(content.clone()),
                    };
                    
                    // Find implicit dependencies in content
                    let reference_regex = &state.reference_regex;
                    for captures in reference_regex.captures_iter(&content) {
                        if let Some(referenced_block) = captures.get(1) {
                            let referenced_name = referenced_block.as_str();
                            requires.insert(referenced_name.to_string());
                        }
                    }
                    
                    // Create the block entity
                    let block_entity = commands.spawn(Block {
                        block_type: block_type.clone(),
                        name: name.clone(),
                        content: block_content,
                        modifiers,
                        depends_on: depends_on.clone(),
                        requires: requires.clone(),
                        fallback: fallback.clone(),
                        executed: false,
                        output: None,
                        error: None,
                    }).id();
                    
                    // Register named blocks
                    if let Some(name_str) = name.clone() {
                        let mut named_blocks = world.get_resource_mut::<NamedBlocks>().unwrap();
                        
                        // Check for name conflicts
                        if named_blocks.0.contains_key(&name_str) {
                            return Err(Error::NamespaceConflict(name_str));
                        }
                        
                        named_blocks.0.insert(name_str, BlockId(block_entity));
                    }
                    
                    // Register dependencies
                    let mut execution_state = world.get_resource_mut::<ExecutionState>().unwrap();
                    if let Some(name_str) = &name {
                        // Combine explicit and implicit dependencies
                        let mut all_deps = HashSet::new();
                        all_deps.extend(depends_on.iter().cloned());
                        all_deps.extend(requires.iter().cloned());
                        
                        execution_state.dependencies.insert(name_str.clone(), all_deps);
                    }
                    
                    // Send event that block was parsed
                    let mut ev_writer = world.get_resource_mut::<Events<BlockParsedEvent>>().unwrap();
                    ev_writer.send(BlockParsedEvent {
                        block_id: BlockId(block_entity),
                        name: name.clone(),
                    });
                    
                    // Add to execution queue if appropriate
                    if block_type.requires_fallback() || matches!(block_type, BlockType::Question) {
                        execution_state.queue.push_back(BlockId(block_entity));
                    }
                    
                    // Move past this block
                    state.current_position = content_end + end_tag.len();
                } else {
                    // Couldn't find end tag
                    return Err(Error::ParseError {
                        pos: end_pos,
                        message: format!("Could not find closing tag for block: [{}]", block_type_str),
                    });
                }
            } else {
                // Unknown block type
                return Err(Error::ParseError {
                    pos: start_pos,
                    message: format!("Unknown block type: {}", block_type_str),
                });
            }
        } else {
            // No more blocks found
            state.is_complete = true;
        }
        
        Ok(())
    }
    
    /// Validates the document after parsing
    fn validate_document(&self, world: &mut World) -> Result<()> {
        // Validate dependencies and fallbacks
        let config = world.resource::<ParserConfig>().clone();

        {
            let named_blocks = world.resource::<NamedBlocks>();
            let execution_state = world.resource::<ExecutionState>();
    
            if !config.allow_circular_dependencies {
                // ... dependency checks ...
            }
    
            // ... dependency existence checks ...
        }

        {
            let named_blocks = world.resource::<NamedBlocks>();
            let block_entities: Vec<(Entity, BlockType, Option<String>, Option<String>)> = world
                .query::<(Entity, &Block)>()
                .iter(world)
                .filter_map(|(entity, block)| {
                    if block.block_type.requires_fallback() {
                        Some((entity, block.block_type.clone(), block.name.clone(), block.fallback.clone()))
                    } else {
                        None
                    }
                })
                .collect();
        
            for (entity, block_type, name, fallback) in block_entities {
                if let Some(fallback_name) = fallback {
                    if !named_blocks.0.contains_key(&fallback_name) {
                        return Err(Error::MissingFallback(
                            fallback_name,
                            name.unwrap_or_else(|| entity.to_string())
                        ));
                    }
                } else {
                    return Err(Error::MissingFallback(
                        "None".to_string(),
                        name.unwrap_or_else(|| entity.to_string())
                    ));
                }
            }
        }
        
        // Check for circular dependencies
        if !config.allow_circular_dependencies {
            for (block_name, deps) in &execution_state.dependencies {
                for dep_name in deps {
                    if let Some(dep_deps) = execution_state.dependencies.get(dep_name) {
                        if dep_deps.contains(block_name) {
                            return Err(Error::CircularDependency(
                                block_name.clone(), 
                                dep_name.clone()
                            ));
                        }
                    }
                }
            }
        }
        
        // Check that all dependencies exist
        for (block_name, deps) in &execution_state.dependencies {
            for dep_name in deps {
                if !named_blocks.0.contains_key(dep_name) {
                    return Err(Error::MissingDependency(
                        block_name.clone(),
                        dep_name.clone()
                    ));
                }
            }
        }
        
        // Check that all executable blocks have fallbacks
        let block_query = world.query::<(Entity, &Block)>();
        for (entity, block) in block_query.iter(world) {
            if block.block_type.requires_fallback() {
                if let Some(fallback_name) = &block.fallback {
                    if !named_blocks.0.contains_key(fallback_name) {
                        return Err(Error::MissingFallback(
                            fallback_name.clone(),
                            block.name.clone().unwrap_or_else(|| entity.to_string())
                        ));
                    }
                } else {
                    return Err(Error::MissingFallback(
                        "None".to_string(),
                        block.name.clone().unwrap_or_else(|| entity.to_string())
                    ));
                }
            }
        }
        
        Ok(())
    }
}

/// Plugin that adds the document parser to the app
// derive clone + copy
#[derive(Clone)]
pub struct ParserPlugin {
    pub document: String,
    pub config: ParserConfig,
}

impl ParserPlugin {
    pub fn new(document: String) -> Self {
        Self {
            document,
            config: ParserConfig::default(),
        }
    }
    
    pub fn with_config(mut self, config: ParserConfig) -> Self {
        self.config = config;
        self
    }
}

impl Plugin for ParserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NamedBlocks>()
           .init_resource::<ExecutionState>()
           .init_resource::<ProcessorRegistry>()
           .insert_resource(self.config.clone())
           .insert_resource(DocumentParser::new(self.document.clone()))
           .add_event::<BlockParsedEvent>()
           .add_event::<BlockExecutedEvent>()
           .add_event::<BlockExecutionRequestEvent>()
           .add_systems(Startup, parse_document)
           .add_systems(Update, (
               on_block_parsed,
               execute_blocks,
               process_executed_blocks,
           ));
    }
}

/// System that parses the document
fn parse_document(
    document_parser: Res<DocumentParser>,
    mut commands: Commands,
) {
    // Initialize the world with parsing state
    // The actual parsing will happen in the on_startup system
    let mut world = World::new();
    world.insert_resource(NamedBlocks::default());
    world.insert_resource(ExecutionState::default());
    
    // Parse the document
    if let Err(err) = document_parser.parse(&mut world) {
        error!("Document parsing failed: {}", err);
    }
    
    // TODO: In a real implementation, we would transfer the parsed entities 
    // from the temporary world to the app's world
}

/// System that reacts to blocks being parsed
fn on_block_parsed(
    mut ev_reader: EventReader<BlockParsedEvent>,
    named_blocks: Res<NamedBlocks>,
    mut execution_state: ResMut<ExecutionState>,
    registry: Res<ProcessorRegistry>,
    mut commands: Commands,
    world: &mut World,
) {
    for event in ev_reader.read() {
        let entity = event.block_id.entity();
        
        // Find the block's type
        if let Some(block) = world.get::<Block>(entity) {
            let block_type = block.block_type.as_str();
            
            // Call the appropriate processor's on_parse method
            if let Some(processor) = registry.get(&block_type) {
                let mut ctx = BlockProcessorContext {
                    commands: &mut commands,
                    named_blocks: &named_blocks,
                    execution_state: &mut execution_state,
                    world,
                };
                
                if let Err(err) = processor.on_parse(event.block_id, &mut ctx) {
                    error!("Block on_parse failed: {}", err);
                }
            }
        }
    }
}

/// System that executes blocks in the execution queue
fn execute_blocks(
    mut execution_state: ResMut<ExecutionState>,
    mut ev_writer: EventWriter<BlockExecutedEvent>,
    registry: Res<ProcessorRegistry>,
    named_blocks: Res<NamedBlocks>,
    block_query: Query<&Block>,
    mut commands: Commands,
    world: &mut World,
) {
    // Process the execution queue
    while let Some(block_id) = execution_state.queue.pop_front() {
        // Skip if already executed or pending
        if execution_state.executed.contains(&block_id) || 
           execution_state.pending.contains(&block_id) {
            continue;
        }
        
        let entity = block_id.entity();
        
        // Get the block
        if let Ok(block) = block_query.get(entity) {
            // Check if all dependencies are satisfied
            let mut dependencies_satisfied = true;
            
            if let Some(name) = &block.name {
                if let Some(deps) = execution_state.dependencies.get(name) {
                    for dep in deps {
                        if let Some(&dep_id) = named_blocks.0.get(dep) {
                            if !execution_state.executed.contains(&dep_id) {
                                dependencies_satisfied = false;
                                break;
                            }
                        } else {
                            // Dependency not found
                            dependencies_satisfied = false;
                            break;
                        }
                    }
                }
            }
            
            if dependencies_satisfied {
                // Find the appropriate processor
                let block_type = block.block_type.as_str();
                
                if let Some(processor) = registry.get(&block_type) {
                    // Create context for processor
                    let mut ctx = BlockProcessorContext {
                        commands: &mut commands,
                        named_blocks: &named_blocks,
                        execution_state: &mut execution_state,
                        world,
                    };
                    
                    // Process the block
                    match processor.process(block_id, &mut ctx) {
                        Ok(output) => {
                            // Update block status
                            if let Some(mut block) = commands.get_entity(entity).and_then(|e| e.get_mut::<Block>()) {
                                block.executed = true;
                                block.output = output.clone();
                            }
                            
                            // Send execution event
                            ev_writer.send(BlockExecutedEvent {
                                block_id,
                                output,
                                error: None,
                            });
                            
                            // Mark as executed
                            execution_state.executed.insert(block_id);
                        },
                        Err(err) => {
                            // Try fallback if available
                            if let Some(fallback_name) = &block.fallback {
                                if let Some(&fallback_id) = named_blocks.0.get(fallback_name) {
                                    // Queue fallback for execution
                                    execution_state.queue.push_front(fallback_id);
                                    
                                    // Update block status
                                    commands.entity(entity).update(|block: &mut Block| {
                                        block.error = Some(err.to_string());
                                    });
                                }
                            } else {
                                // No fallback, report error
                                ev_writer.send(BlockExecutedEvent {
                                    block_id,
                                    output: None,
                                    error: Some(err.to_string()),
                                });
                                
                                // Mark as executed with error
                                execution_state.executed.insert(block_id);
                                
                                commands.entity(entity).update(|block: &mut Block| {
                                    block.error = Some(err.to_string());
                                });
                            }
                        }
                    }
                } else {
                    // No processor for this block type
                    error!("No processor found for block type: {}", block_type);
                    
                    // Mark as executed
                    execution_state.executed.insert(block_id);
                }
            } else {
                // Dependencies not satisfied, put back in queue
                execution_state.queue.push_back(block_id);
            }
        }
    }
}

/// System that processes executed blocks
fn process_executed_blocks(
    mut ev_reader: EventReader<BlockExecutedEvent>,
    mut commands: Commands,
) {
    for event in ev_reader.read() {
        let entity = event.block_id.entity();
        
        if let Some(output) = &event.output {
            // Update the block with the execution result
            commands.entity(entity).update(|block: &mut Block| {
                block.output = Some(output.clone());
                block.executed = true;
            });
        } else if let Some(error) = &event.error {
            // Update the block with the error
            commands.entity(entity).update(|block: &mut Block| {
                block.error = Some(error.clone());
                block.executed = true;
            });
        }
    }
}

// Implement default processors for common block types

/// Code block processor
pub struct CodeBlockProcessor;

impl BlockProcessor for CodeBlockProcessor {
    fn block_type(&self) -> String {
        "code".to_string()
    }
    
    fn process(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<Option<BlockContent>> {
        let entity = block_id.entity();
        
        if let Some(block) = ctx.world.get::<Block>(entity) {
            // In a real implementation, this would execute the code
            if let BlockContent::Code { language, source } = &block.content {
                info!("Executing {} code: {}", language, source);
                
                // Simulate code execution
                let result = format!("Result of {} code execution", language);
                
                Ok(Some(BlockContent::Text(result)))
            } else {
                Err(Error::ExecutionError("Block content is not code".to_string()))
            }
        } else {
            Err(Error::ExecutionError("Block not found".to_string()))
        }
    }
}

/// Data block processor
pub struct DataBlockProcessor;

impl BlockProcessor for DataBlockProcessor {
    fn block_type(&self) -> String {
        "data".to_string()
    }
    
    fn process(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<Option<BlockContent>> {
        let entity = block_id.entity();
        
        if let Some(block) = ctx.world.get::<Block>(entity) {
            // Data blocks don't need processing, just return the content
            Ok(Some(block.content.clone()))
        } else {
            Err(Error::ExecutionError("Block not found".to_string()))
        }
    }
}

/// Question block processor
pub struct QuestionBlockProcessor;

impl BlockProcessor for QuestionBlockProcessor {
    fn block_type(&self) -> String {
        "question".to_string()
    }
    
    fn process(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<Option<BlockContent>> {
        let entity = block_id.entity();
        
        if let Some(block) = ctx.world.get::<Block>(entity) {
            // In a real implementation, this would call an AI model
            if let BlockContent::Text(question) = &block.content {
                info!("Processing question: {}", question);
                
                // Simulate AI response
                let default_model = "default".to_string();
                let model = block.modifiers.get("model").unwrap_or(&default_model);
                let response = format!("AI response using model {} to question: {}", model, question);
                
                // Create a response block
                if let Some(name) = &block.name {
                    let response_name = format!("{}-response", name);
                    
                    let response_entity = ctx.commands.spawn(Block {
                        block_type: BlockType::Response,
                        name: Some(response_name.clone()),
                        content: BlockContent::Text(response.clone()),
                        modifiers: HashMap::new(),
                        depends_on: {
                            let mut deps = HashSet::new();
                            deps.insert(name.clone());
                            deps
                        },
                        requires: HashSet::new(),
                        fallback: None,
                        executed: true,
                        output: None,
                        error: None,
                    }).id();
                    
                    // Register the response block
                    ctx.named_blocks.0.insert(response_name, BlockId(response_entity));
                }
                
                Ok(Some(BlockContent::Text(response)))
            } else {
                Err(Error::ExecutionError("Block content is not text".to_string()))
            }
        } else {
            Err(Error::ExecutionError("Block not found".to_string()))
        }
    }
}

/// Shell block processor
pub struct ShellBlockProcessor;

impl BlockProcessor for ShellBlockProcessor {
    fn block_type(&self) -> String {
        "shell".to_string()
    }
    
    fn process(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<Option<BlockContent>> {
        let entity = block_id.entity();
        
        if let Some(block) = ctx.world.get::<Block>(entity) {
            if let BlockContent::Shell(command) = &block.content {
                info!("Executing shell command: {}", command);
                
                // In a real implementation, this would execute the shell command
                // For now, we'll just simulate it
                let timeout = block.modifiers.get("timeout")
                    .and_then(|t| t.parse::<u64>().ok())
                    .unwrap_or(30);
                
                let result = format!("Result of shell command (timeout: {}s): {}", timeout, command);
                
                Ok(Some(BlockContent::Text(result)))
            } else {
                Err(Error::ExecutionError("Block content is not a shell command".to_string()))
            }
        } else {
            Err(Error::ExecutionError("Block not found".to_string()))
        }
    }
}

/// API block processor
pub struct ApiBlockProcessor;

impl BlockProcessor for ApiBlockProcessor {
    fn block_type(&self) -> String {
        "api".to_string()
    }
    
    fn process(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<Option<BlockContent>> {
        let entity = block_id.entity();
        
        if let Some(block) = ctx.world.get::<Block>(entity) {
            if let BlockContent::Api { url, method, headers } = &block.content {
                info!("Making API request: {} {}", method, url);
                
                // In a real implementation, this would make the API request
                // For now, we'll just simulate it
                let retry = block.modifiers.get("retry")
                    .and_then(|r| r.parse::<u32>().ok())
                    .unwrap_or(0);
                
                let result = format!("API response from {} (retries: {}): {{\n  \"status\": \"success\",\n  \"data\": {{}}\n}}", url, retry);
                
                // Try to parse as JSON if it looks like JSON
                if result.trim().starts_with('{') || result.trim().starts_with('[') {
                    if let Ok(json) = serde_json::from_str(&result) {
                        return Ok(Some(BlockContent::Json(json)));
                    }
                }
                
                Ok(Some(BlockContent::Text(result)))
            } else {
                Err(Error::ExecutionError("Block content is not an API request".to_string()))
            }
        } else {
            Err(Error::ExecutionError("Block not found".to_string()))
        }
    }
}

/// Template block processor
pub struct TemplateBlockProcessor;

impl BlockProcessor for TemplateBlockProcessor {
    fn block_type(&self) -> String {
        "template".to_string()
    }
    
    fn process(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<Option<BlockContent>> {
        // Templates don't need direct execution, they're used when invoked
        Ok(None)
    }
    
    fn on_parse(&self, block_id: BlockId, ctx: &mut BlockProcessorContext) -> Result<()> {
        // When a template is parsed, we register it for later use
        info!("Template registered: {:?}", block_id);
        Ok(())
    }
}

// Plugins for each block processor

/// Plugin for the code block processor
pub struct CodeBlockProcessorPlugin;

impl Plugin for CodeBlockProcessorPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app.world_mut().resource_mut::<ProcessorRegistry>();
        registry.register(CodeBlockProcessor);
    }
}

impl BlockProcessorPlugin for CodeBlockProcessorPlugin {
    fn processor(&self) -> Box<dyn BlockProcessor + Send + Sync> {
        Box::new(CodeBlockProcessor)
    }
}

/// Plugin for the data block processor
pub struct DataBlockProcessorPlugin;

impl Plugin for DataBlockProcessorPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app.world.resource_mut::<ProcessorRegistry>();
        registry.register(DataBlockProcessor);
    }
}

impl BlockProcessorPlugin for DataBlockProcessorPlugin {
    fn processor(&self) -> Box<dyn BlockProcessor + Send + Sync> {
        Box::new(DataBlockProcessor)
    }
}

/// Plugin for the question block processor
pub struct QuestionBlockProcessorPlugin;

impl Plugin for QuestionBlockProcessorPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app.world.resource_mut::<ProcessorRegistry>();
        registry.register(QuestionBlockProcessor);
    }
}

impl BlockProcessorPlugin for QuestionBlockProcessorPlugin {
    fn processor(&self) -> Box<dyn BlockProcessor + Send + Sync> {
        Box::new(QuestionBlockProcessor)
    }
}

/// Plugin for the shell block processor
pub struct ShellBlockProcessorPlugin;

impl Plugin for ShellBlockProcessorPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app.world.resource_mut::<ProcessorRegistry>();
        registry.register(ShellBlockProcessor);
    }
}

impl BlockProcessorPlugin for ShellBlockProcessorPlugin {
    fn processor(&self) -> Box<dyn BlockProcessor + Send + Sync> {
        Box::new(ShellBlockProcessor)
    }
}

/// Plugin for the API block processor
pub struct ApiBlockProcessorPlugin;

impl Plugin for ApiBlockProcessorPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app.world.resource_mut::<ProcessorRegistry>();
        registry.register(ApiBlockProcessor);
    }
}

impl BlockProcessorPlugin for ApiBlockProcessorPlugin {
    fn processor(&self) -> Box<dyn BlockProcessor + Send + Sync> {
        Box::new(ApiBlockProcessor)
    }
}

/// Plugin for the template block processor
pub struct TemplateBlockProcessorPlugin;

impl Plugin for TemplateBlockProcessorPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app.world.resource_mut::<ProcessorRegistry>();
        registry.register(TemplateBlockProcessor);
    }
}

impl BlockProcessorPlugin for TemplateBlockProcessorPlugin {
    fn processor(&self) -> Box<dyn BlockProcessor + Send + Sync> {
        Box::new(TemplateBlockProcessor)
    }
}

/// Plugin that adds all standard block processors
pub struct StandardProcessorsPlugin;

impl Plugin for StandardProcessorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CodeBlockProcessorPlugin,
            DataBlockProcessorPlugin,
            QuestionBlockProcessorPlugin,
            ShellBlockProcessorPlugin,
            ApiBlockProcessorPlugin,
            TemplateBlockProcessorPlugin,
        ));
    }
}

// Example of how to use the library
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_document() {
        let mut app = App::new();
        
        let document = r#"
        [data name:sample-data format:json]
        {
            "key": "value",
            "number": 42
        }
        [/data]
        
        [question name:analyze-data]
        Analyze this data: ${sample-data}
        [/question]
        "#;
        
        app.add_plugins(MinimalPlugins)
           .add_plugins(ParserPlugin::new(document.to_string()))
           .add_plugins(StandardProcessorsPlugin);
        
        app.update();
        
        let named_blocks = app.world.resource::<NamedBlocks>();
        let execution_state = app.world.resource::<ExecutionState>();
        
        assert!(named_blocks.0.contains_key("sample-data"), "sample-data block should exist");
        assert!(named_blocks.0.contains_key("analyze-data"), "analyze-data block should exist");
        
        assert!(execution_state.dependencies.contains_key("analyze-data"), 
               "analyze-data should have dependencies");
        
        if let Some(deps) = execution_state.dependencies.get("analyze-data") {
            assert!(deps.contains("sample-data"), "analyze-data should depend on sample-data");
        }
    }
    
    #[test]
    fn test_execute_code_block() {
        let mut app = App::new();
        
        let document = r#"
        [code:python name:sample-code fallback:sample-fallback]
        print("Hello, World!")
        [/code:python]
        
        [code:python name:sample-fallback]
        print("Fallback called")
        [/code:python]
        "#;
        
        app.add_plugins(MinimalPlugins)
           .add_plugins(ParserPlugin::new(document.to_string()))
           .add_plugins(StandardProcessorsPlugin);
        
        // Run a few update cycles to ensure execution completes
        for _ in 0..5 {
            app.update();
        }
        
        let named_blocks = app.world.resource::<NamedBlocks>();
        let execution_state = app.world.resource::<ExecutionState>();
        
        assert!(named_blocks.0.contains_key("sample-code"), "sample-code block should exist");
        assert!(named_blocks.0.contains_key("sample-fallback"), "sample-fallback block should exist");
        
        let sample_code_id = named_blocks.0.get("sample-code").unwrap();
        assert!(execution_state.executed.contains(sample_code_id), "sample-code should be executed");
        
        let entity = sample_code_id.entity();
        let block_query = app.world.query::<&Block>();
        let block = block_query.get(&app.world, entity).unwrap();
        
        assert!(block.executed, "Block should be marked as executed");
        assert!(block.output.is_some(), "Block should have output");
    }
}
