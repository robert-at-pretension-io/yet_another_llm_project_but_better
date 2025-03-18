use std::io;
use quick_xml::events::attributes::AttrError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Circular dependency: {0}")]
    CircularDependency(String),

    #[error("Missing fallback: {0}")]
    MissingFallback(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("XML attribute error: {0}")]
    XmlAttributeError(#[from] AttrError),

    #[error("LLM API error: {0}")]
    LlmApiError(String),

    #[error("Missing API key: {0}")]
    MissingApiKey(String),

    #[error("Failed to resolve reference: {0}")]
    ReferenceResolutionFailed(String),

    #[error("XML parsing error: {0}")]
    XmlParsingError(String),
}
