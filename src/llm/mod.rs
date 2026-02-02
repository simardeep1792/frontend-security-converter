//! LLM Provider abstraction layer
//!
//! This module provides a unified interface for multiple LLM providers,
//! allowing the application to switch between Ollama and Google Gemini.

pub mod provider;
pub mod ollama_provider;
pub mod gemini_provider;

pub use provider::{LlmProvider, LlmError, LlmResponse};
pub use ollama_provider::OllamaProvider;
pub use gemini_provider::GeminiProvider;

use std::sync::Arc;

/// Enum to hold the active LLM provider
#[derive(Clone)]
pub enum LlmClient {
    Ollama(Arc<OllamaProvider>),
    Gemini(Arc<GeminiProvider>),
}

impl LlmClient {
    /// Generate a response from the LLM with the given prompt
    pub async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<LlmResponse, LlmError> {
        match self {
            LlmClient::Ollama(provider) => provider.generate(prompt, model).await,
            LlmClient::Gemini(provider) => provider.generate(prompt, model).await,
        }
    }

    /// Get the provider name for logging purposes
    pub fn provider_name(&self) -> &'static str {
        match self {
            LlmClient::Ollama(_) => "Ollama",
            LlmClient::Gemini(_) => "Gemini",
        }
    }
}
