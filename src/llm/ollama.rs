use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    format: serde_json::Value,
    stream: bool,
    think: bool,
    keep_alive: String,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

pub async fn extract_metadata_with_ollama(
    client: &Client,
    document_content: &str,
    selected_countries: &[String],
    selected_organizations: &[String],
    selected_handling_restrictions: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    let host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://10.0.0.14".to_string());
    let port = env::var("OLLAMA_PORT").unwrap_or_else(|_| "11434".to_string());
    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:14b".to_string());
    let keep_alive = env::var("OLLAMA_KEEP_ALIVE").unwrap_or_else(|_| "1h".to_string());
    let temperature = env::var("OLLAMA_TEMPERATURE")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.1);
    let top_p = env::var("OLLAMA_TOP_P")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.9);
    let num_predict = env::var("OLLAMA_NUM_PREDICT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(320);

    let endpoint = format!("{}:{}/api/generate", host.trim_end_matches('/'), port);

    let countries_hint = if selected_countries.is_empty() {
        "[]".to_string()
    } else {
        serde_json::to_string(selected_countries)?
    };
    let organizations_hint = if selected_organizations.is_empty() {
        "[]".to_string()
    } else {
        serde_json::to_string(selected_organizations)?
    };
    let handling_hint = if selected_handling_restrictions.is_empty() {
        "[]".to_string()
    } else {
        serde_json::to_string(selected_handling_restrictions)?
    };

    let prompt = format!(
        "Return ONLY one minified JSON object with these keys exactly: \
        title,description,domain,tags,identifier,authorization_reference,releasable_to_categories.\n\
        Constraints: domain in [INTEL,CYBER,OPERATIONS,LOGISTICS,COMMUNICATIONS,NUCLEAR,COUNTERTERRORISM,MARITIME,AEROSPACE,SPECIALOPS]. \
        tags must be 3-6 short uppercase strings. \
        identifier format ORG-DOMAIN-DATE-XXXX. \
        If unknown, use null for nullable fields and [] for list fields.\n\
        Operator constraints (high priority):\n\
        selected_countries={}\n\
        selected_organizations={}\n\
        selected_handling_restrictions={}\n\
        These lists are operator-selected controls handled by the web form; do not invent alternatives.\n\
        Document:\n{}",
        countries_hint,
        organizations_hint,
        handling_hint,
        document_content
    );

    let payload = OllamaGenerateRequest {
        model,
        prompt,
        format: json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "description": {"type": "string"},
                "domain": {"type": "string"},
                "tags": {"type": "array", "items": {"type": "string"}},
                "identifier": {"type": "string"},
                "authorization_reference": {"type": ["string", "null"]},
                "releasable_to_categories": {"type": "array", "items": {"type": ["string", "null"]}},
                "disclosure_category": {"type": ["string", "null"]}
            },
            "required": [
                "title","description","domain","tags","identifier",
                "authorization_reference","releasable_to_categories"
            ],
            "additionalProperties": false
        }),
        stream: false,
        think: false,
        keep_alive,
        options: OllamaOptions {
            temperature,
            top_p,
            num_predict,
        },
    };

    let response = client
        .post(endpoint)
        .json(&payload)
        .send()
        .await?
        .json::<OllamaGenerateResponse>()
        .await?;

    Ok(response.response)
}
