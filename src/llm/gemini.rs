use reqwest::Client;
use serde_json::{json, Value};
use std::env;

/// Fetches a short-lived GCP access token from the GKE metadata server.
/// Works automatically when Workload Identity is configured on the pod.
/// No API keys required.
async fn fetch_access_token(client: &Client) -> Result<String, reqwest::Error> {
    let resp = client
        .get(
            "http://metadata.google.internal/computeMetadata/v1/instance/\
             service-accounts/default/token",
        )
        .header("Metadata-Flavor", "Google")
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(resp["access_token"].as_str().unwrap_or_default().to_string())
}

/// Calls Gemini 1.5 Flash via Vertex AI to extract structured metadata
/// from a raw document string. Returns the raw JSON string Gemini produces.
/// The caller deserializes this into LLMFields.
pub async fn extract_metadata_with_gemini(
    client: &Client,
    document_content: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let project_id = env::var("GEMINI_PROJECT_ID").expect("GEMINI_PROJECT_ID env var must be set");
    let location = env::var("GEMINI_LOCATION").unwrap_or_else(|_| "northamerica-northeast1".to_string());
    let model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".to_string());

    let token = fetch_access_token(client).await?;

    let prompt = format!(
        "Extract security classification metadata from this document as valid JSON.\n\n\
        Document:\n{}\n\n\
        Requirements:\n\
        - title: Clear descriptive title (required string)\n\
        - description: 2-sentence summary (required string)\n\
        - domain: One of INTEL, CYBER, OPERATIONS, LOGISTICS, COMMUNICATIONS, \
          NUCLEAR, COUNTERTERRORISM, MARITIME, AEROSPACE, SPECIALOPS\n\
        - tags: Array of 3-6 classification tags (required, array of strings)\n\
        - identifier: Unique ID in format ORG-DOMAIN-DATE-XXXX (required string)\n\
        - authorization_reference: Reference authority document (string or null)\n\
        - releasable_to_countries: Array of ISO 3-letter country codes or empty array\n\
        - releasable_to_organizations: Array from (NATO, EU, UN, FVEY, AUKUS, QUAD) or empty array\n\
        - releasable_to_categories: Array of strings or empty array\n\
        - disclosure_category: String or null\n\
        - handling_restrictions: Array from (CUI, FOUO, LES, SBU, NOFORN, PROPIN, ORCON) or empty array\n\
        - handling_authority: String or null\n\
        - no_handling_restrictions: Boolean or null\n\n\
        Output ONLY valid JSON. No explanation. No markdown fences.",
        document_content
    );

    let endpoint = format!(
        "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/\
         publishers/google/models/{}:generateContent",
        location, project_id, location, model
    );

    let body = json!({
        "contents": [{
            "role": "user",
            "parts": [{ "text": prompt }]
        }],
        "generationConfig": {
            "responseMimeType": "application/json",
            "temperature": 0.1,
            "topP": 0.9,
            "maxOutputTokens": 2048
        }
    });

    let response = client
        .post(&endpoint)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let content = response["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("{}")
        .to_string();

    Ok(content)
}
