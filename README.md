# Frontend Security Converter

A Rust web application that serves as a frontend trigger for the [Security Converter API](https://github.com/ToferC/security_converter). The application uses LLM-powered metadata extraction to process security classification conversion requests.

**Live Demo:** https://security-converter.obrienlabs.dev/

## Features

- **User Authentication** - Azure SDK integration for user authentication
- **Authority Validation** - Validates users and their authority with the backend API
- **LLM-Powered Metadata Extraction** - Uses AI to automatically extract security metadata from documents
- **Multiple LLM Providers** - Supports both Ollama (local) and Google Gemini (cloud)
- **Bilingual Support** - Full English/French internationalization via Fluent
- **Conversion Request Processing** - Submits and tracks security classification conversions

## Technology Stack

- **Rust** (Edition 2021) with **Actix-Web 4** web framework
- **Tera** templating engine for HTML rendering
- **GraphQL** client for API integration
- **Fluent Templates** for i18n (EN/FR)
- **Bootstrap** + **jQuery** for frontend styling
- **Ollama** or **Google Gemini** for LLM processing

## Quick Start

### Prerequisites

- Rust (latest stable)
- An LLM provider:
  - **Ollama** (recommended for local development): [Install Ollama](https://ollama.ai)
  - **Google Gemini**: [Get API Key](https://makersuite.google.com/app/apikey)

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-repo/frontend-security-converter.git
   cd frontend-security-converter
   ```

2. Create your environment file:
   ```bash
   cp .env.example .env
   ```

3. Configure your LLM provider (see [LLM Configuration](#llm-configuration) below)

4. Run the application:
   ```bash
   cargo run
   ```

5. Open http://127.0.0.1:8088 in your browser

## Environment Variables

### Core Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `COOKIE_SECRET_KEY` | Session encryption key (min 32 chars) | Required |
| `ENVIRONMENT` | `test` or `production` | `test` |

### Production Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server bind address | `127.0.0.1` |
| `PORT` | Server port | `8088` |
| `GRAPHQL_API_TARGET` | Backend API endpoint | `http://127.0.0.1:8080/graphql` |

### LLM Configuration

The application supports two LLM providers for extracting security metadata from documents:

#### Option 1: Ollama (Default - Local/Self-Hosted)

[Ollama](https://ollama.ai) runs LLMs locally on your machine or server.

```bash
# Install Ollama and pull a model
ollama pull llama3:8b

# Configure in .env
LLM_PROVIDER=ollama
OLLAMA_HOST=http://localhost
OLLAMA_PORT=11434
OLLAMA_MODEL=llama3:8b
```

**Recommended Models:**
| Model | Speed | Accuracy | VRAM Required |
|-------|-------|----------|---------------|
| `llama3:8b` | Balanced | Good | ~8GB |
| `mistral:7b` | Fast | Good | ~6GB |
| `gemma2:27b` | Slow | Best | ~20GB |

#### Option 2: Google Gemini (Cloud)

[Google Gemini](https://ai.google.dev/) provides cloud-based LLM access.

```bash
# Configure in .env
LLM_PROVIDER=gemini
GEMINI_API_KEY=your-api-key-here
GEMINI_MODEL=gemini-1.5-flash
```

**Available Models:**
| Model | Description |
|-------|-------------|
| `gemini-1.5-flash` | Fast, cost-effective |
| `gemini-1.5-pro` | More capable, higher cost |

## Development Commands

```bash
# Build the application
cargo build

# Run in development mode
cargo run

# Check for compilation errors
cargo check

# Run linting
cargo clippy

# Run tests
cargo test
```

## Project Structure

```
frontend-security-converter/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library definitions, AppData
│   ├── handlers/            # HTTP request handlers
│   │   ├── conversion_response.rs  # LLM integration for document processing
│   │   └── ...
│   ├── graphql/             # GraphQL client queries
│   ├── llm/                 # LLM provider abstraction
│   │   ├── mod.rs           # LlmClient enum
│   │   ├── provider.rs      # LlmProvider trait
│   │   ├── ollama_provider.rs
│   │   └── gemini_provider.rs
│   └── models/              # Data models
├── templates/               # Tera HTML templates
├── static/                  # CSS, JS, assets
├── i18n/                    # Internationalization (EN/FR)
├── queries/                 # GraphQL query definitions
└── deploy/                  # Deployment scripts
```

## LLM Integration

The application uses LLMs to automatically extract security metadata from submitted documents. The extraction includes:

- **Title** - Document title
- **Description** - Two-sentence summary
- **Domain** - Security domain (INTEL, CYBER, OPERATIONS, etc.)
- **Tags** - Classification tags for searchability
- **Identifier** - Unique document identifier
- **Authorization Reference** - Release authority documentation
- **Handling Restrictions** - Security handling requirements

### Adding a New LLM Provider

To add support for a new LLM provider:

1. Create a new file in `src/llm/` (e.g., `anthropic_provider.rs`)
2. Implement the `LlmProvider` trait:
   ```rust
   impl LlmProvider for YourProvider {
       async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<LlmResponse, LlmError>;
       fn default_model(&self) -> &str;
       fn provider_name(&self) -> &'static str;
   }
   ```
3. Add the provider to `LlmClient` enum in `src/llm/mod.rs`
4. Update `src/main.rs` to handle the new provider selection

## Deployment

### GCP VM Deployment

Deployment scripts are provided for deploying to a GCP VM:

```bash
# From Windows (PowerShell)
./deploy/deploy-to-gcp.ps1

# From Linux/Mac
./deploy/deploy-to-gcp.sh
```

The scripts will:
1. Create a tarball of the source code
2. Copy to the VM via IAP tunnel
3. Build and install as a systemd service

### Production Environment Variables

```bash
ENVIRONMENT=production
HOST=0.0.0.0
PORT=8088
GRAPHQL_API_TARGET=http://your-api-endpoint/graphql
LLM_PROVIDER=gemini  # or ollama
GEMINI_API_KEY=your-key  # if using Gemini
```

## Roadmap

- [x] Actix-Web with async
- [x] Tera templates
- [x] Authentication and sign-in
- [x] Static files compilation
- [x] Fluent i18n integration
- [x] LLM integration (Ollama)
- [x] Multiple LLM provider support (Gemini)
- [ ] Azure SDK authentication
- [ ] MCP connections for LLM

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]
