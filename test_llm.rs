// Simple test script - run with: cargo run --example test_llm
// Or copy this into a standalone project

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

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
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let api_key = env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY not set in .env");
    let model = env::var("GEMINI_MODEL")
        .unwrap_or_else(|_| "gemini-1.5-flash".to_string());

    println!("Testing Gemini API with model: {}", model);

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let test_prompt = r#"Extract security metadata from this document as valid JSON.

Document: This is a classified intelligence report regarding cyber operations in Eastern Europe. The document covers signals intelligence gathered from multiple sources and should be handled with appropriate security protocols.

Output valid JSON with these fields:
- title: string
- description: string
- domain: one of INTEL, CYBER, OPERATIONS
- tags: array of strings

Output JSON only:"#;

    let request_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: test_prompt.to_string(),
            }],
        }],
        generation_config: GenerationConfig {
            temperature: 0.2,
            max_output_tokens: 1024,
        },
    };

    let client = Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    println!("\nStatus: {}", status);

    if status.is_success() {
        let gemini_response: GeminiResponse = serde_json::from_str(&text)?;
        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                println!("\nLLM Response:\n{}", part.text);
            }
        }
    } else {
        println!("\nError response:\n{}", text);
    }

    Ok(())
}
