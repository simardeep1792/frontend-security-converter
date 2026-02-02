//! Common traits and types for LLM providers

use std::fmt;

/// Error type for LLM operations
#[derive(Debug)]
pub enum LlmError {
    /// Error during generation
    GenerationError(String),
    /// Connection/network error
    ConnectionError(String),
    /// JSON parsing error
    ParseError(String),
    /// Configuration error
    ConfigError(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::GenerationError(msg) => write!(f, "Generation error: {}", msg),
            LlmError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            LlmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LlmError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

/// Response from an LLM generation request
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// The generated text response
    pub content: String,
    /// Model used for generation
    pub model: String,
    /// Time taken for generation in milliseconds (if available)
    pub duration_ms: Option<u64>,
}

/// Trait defining the interface for LLM providers
#[allow(async_fn_in_trait)]
pub trait LlmProvider: Send + Sync {
    /// Generate a response from the LLM
    ///
    /// # Arguments
    /// * `prompt` - The prompt to send to the LLM
    /// * `model` - Optional model override (uses default if None)
    ///
    /// # Returns
    /// * `Ok(LlmResponse)` - The generated response
    /// * `Err(LlmError)` - If generation fails
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<LlmResponse, LlmError>;

    /// Get the default model for this provider
    fn default_model(&self) -> &str;

    /// Get the provider name
    fn provider_name(&self) -> &'static str;
}
