//! Ollama LLM Provider implementation

use ollama_rs::{
    generation::{
        completion::request::GenerationRequest,
        parameters::{FormatType, JsonStructure},
    },
    models::ModelOptions,
    Ollama,
};

use super::provider::{LlmError, LlmProvider, LlmResponse};
use crate::handlers::conversion_response::LLMFields;

/// Ollama LLM provider implementation
#[derive(Clone)]
pub struct OllamaProvider {
    client: Ollama,
    default_model: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    ///
    /// # Arguments
    /// * `host` - The Ollama server host URL
    /// * `port` - The Ollama server port
    /// * `default_model` - The default model to use (e.g., "llama3:8b")
    pub fn new(host: String, port: u16, default_model: String) -> Self {
        let client = Ollama::new(host, port);
        Self {
            client,
            default_model,
        }
    }

    /// Create from environment variables
    ///
    /// Uses OLLAMA_HOST, OLLAMA_PORT, and OLLAMA_MODEL environment variables
    pub fn from_env() -> Result<Self, LlmError> {
        let host = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://ollama-service.security-converter.svc.cluster.local".to_string());
        let port = std::env::var("OLLAMA_PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .map_err(|e| LlmError::ConfigError(format!("Invalid OLLAMA_PORT: {}", e)))?;
        let default_model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3:8b".to_string());

        Ok(Self::new(host, port, default_model))
    }
}

impl LlmProvider for OllamaProvider {
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<LlmResponse, LlmError> {
        let model_name = model.unwrap_or(&self.default_model).to_string();

        let data_obj_format = FormatType::StructuredJson(Box::new(JsonStructure::new::<LLMFields>()));

        let request = GenerationRequest::new(model_name.clone(), prompt.to_string())
            .format(data_obj_format)
            .options(
                ModelOptions::default()
                    .temperature(0.2)
                    .top_k(40)
                    .top_p(0.9)
                    .repeat_penalty(1.1)
                    .num_predict(2048),
            );

        let start = std::time::Instant::now();

        let response = self.client
            .generate(request)
            .await
            .map_err(|e| LlmError::GenerationError(format!("Ollama generation failed: {}", e)))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(LlmResponse {
            content: response.response,
            model: model_name,
            duration_ms: Some(duration_ms),
        })
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    fn provider_name(&self) -> &'static str {
        "Ollama"
    }
}
