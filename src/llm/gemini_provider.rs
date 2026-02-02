//! Google Gemini LLM Provider implementation

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::provider::{LlmError, LlmProvider, LlmResponse};

/// Google Gemini LLM provider implementation
#[derive(Clone)]
pub struct GeminiProvider {
    client: Arc<Client>,
    api_key: String,
    default_model: String,
    base_url: String,
}

/// Gemini API request structure
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "topK")]
    top_k: i32,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
}

/// Gemini API response structure
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    #[allow(dead_code)]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Debug, Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Debug, Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<i32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<i32>,
}

/// Gemini API error response
#[derive(Debug, Deserialize)]
struct GeminiErrorResponse {
    error: GeminiError,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    message: String,
    status: Option<String>,
}

impl GeminiProvider {
    /// Create a new Gemini provider
    ///
    /// # Arguments
    /// * `api_key` - Google Cloud API key with Gemini API access
    /// * `default_model` - The default model to use (e.g., "gemini-1.5-flash")
    pub fn new(api_key: String, default_model: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            api_key,
            default_model,
            base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
        }
    }

    /// Create from environment variables
    ///
    /// Uses GEMINI_API_KEY and GEMINI_MODEL environment variables
    pub fn from_env() -> Result<Self, LlmError> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| LlmError::ConfigError("GEMINI_API_KEY environment variable not set".to_string()))?;
        let default_model = std::env::var("GEMINI_MODEL")
            .unwrap_or_else(|_| "gemini-1.5-flash".to_string());

        Ok(Self::new(api_key, default_model))
    }

    fn build_url(&self, model: &str) -> String {
        format!(
            "{}/{}:generateContent?key={}",
            self.base_url, model, self.api_key
        )
    }
}

impl LlmProvider for GeminiProvider {
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<LlmResponse, LlmError> {
        let model_name = model.unwrap_or(&self.default_model).to_string();
        let url = self.build_url(&model_name);

        let request_body = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                temperature: 0.2,
                top_k: 40,
                top_p: 0.9,
                max_output_tokens: 2048,
                response_mime_type: "application/json".to_string(),
            },
        };

        let start = std::time::Instant::now();

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(format!("Failed to connect to Gemini API: {}", e)))?;

        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| LlmError::ParseError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<GeminiErrorResponse>(&response_text) {
                return Err(LlmError::GenerationError(format!(
                    "Gemini API error: {} ({})",
                    error_response.error.message,
                    error_response.error.status.unwrap_or_else(|| status.to_string())
                )));
            }
            return Err(LlmError::GenerationError(format!(
                "Gemini API error: {} - {}",
                status, response_text
            )));
        }

        let gemini_response: GeminiResponse = serde_json::from_str(&response_text)
            .map_err(|e| LlmError::ParseError(format!("Failed to parse Gemini response: {} - Response: {}", e, response_text)))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        let content = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| LlmError::ParseError("No content in Gemini response".to_string()))?;

        Ok(LlmResponse {
            content,
            model: model_name,
            duration_ms: Some(duration_ms),
        })
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    fn provider_name(&self) -> &'static str {
        "Gemini"
    }
}
