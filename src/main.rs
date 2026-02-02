use actix_web::web;
use actix_web::{HttpServer, App, middleware};
use dotenv::dotenv;
use std::env;
use tera::{Tera};
use tera_text_filters::snake_case;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web_static_files::ResourceFiles;
use reqwest::Client;
use std::sync::Arc;

use frontend::handlers;
use frontend::AppData;
use frontend::llm::{LlmClient, OllamaProvider, GeminiProvider, LlmProvider};

use fluent_templates::{FluentLoader, static_loader};
// https://lib.rs/crates/fluent-templates

// Setup for serving static files
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

static_loader! {
    static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::init();

    let environment = env::var("ENVIRONMENT");

    let environment = match environment {
        Ok(v) => v,
        Err(_) => String::from("test"),
    };

    let (host, port) = if environment == "production" {
        (env::var("HOST").unwrap(), env::var("PORT").unwrap())
    } else {
        (String::from("127.0.0.1"), String::from("8088"))
    };

    let api_target = if environment == "production" {
        env::var("GRAPHQL_API_TARGET").unwrap_or_else(|_| "http://security-converter-api.security-converter.svc.cluster.local/graphql".to_string())
    } else {
        String::from("http://127.0.0.1:8080/graphql")
    };

    let cookie_secret = env::var("COOKIE_SECRET_KEY").expect("Unable to find cookie secret key");

    let cookie_secret_key: Key = Key::from(&cookie_secret.as_bytes());

    // Configure templates via Tera

    let mut tera = Tera::new(
        "templates/**/*").unwrap();

    tera.register_filter("snake_case", snake_case);
    tera.full_reload().expect("Error running auto-reload with Tera");
    tera.register_function("fluent", FluentLoader::new(&*LOCALES));

    // Set API target

    let api_url = format!("{}", api_target);
    
    println!("Serving on http://{}:{}", &host, &port);
    println!("Targeting API on {}", &api_url);
    
    // Create Reqwest Client
    let client = Arc::new(Client::new());

    // Create LLM client based on LLM_PROVIDER environment variable
    // Options: "ollama" (default), "gemini"
    let llm_provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string());

    let llm: LlmClient = match llm_provider.to_lowercase().as_str() {
        "gemini" => {
            println!("Initializing Google Gemini LLM provider...");
            let provider = GeminiProvider::from_env()
                .expect("Failed to initialize Gemini provider. Ensure GEMINI_API_KEY is set.");
            println!("Gemini provider initialized with model: {}", provider.default_model());
            LlmClient::Gemini(Arc::new(provider))
        }
        _ => {
            println!("Initializing Ollama LLM provider...");
            let provider = OllamaProvider::from_env()
                .expect("Failed to initialize Ollama provider");
            println!("Ollama provider initialized with model: {}", provider.default_model());
            LlmClient::Ollama(Arc::new(provider))
        }
    };

    // Initialize AppData
    let data = web::Data::new(AppData {
        tmpl: tera,
        api_url: api_url,
        client: client,
        llm: llm,
    });

    HttpServer::new(move || {
        let generated = generate();

        App::new()
            .wrap(middleware::Logger::default())
            .service(ResourceFiles::new(
                "/static", generated,
            ))
            .configure(handlers::configure_services)
            .app_data(data.clone())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(), cookie_secret_key.clone())
                    .cookie_secure(false)
                    .build()
                )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
