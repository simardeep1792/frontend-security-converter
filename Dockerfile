# Build stage
FROM rustlang/rust:nightly-bookworm AS builder

# Install build dependencies including PostgreSQL client libraries
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock build.rs ./

# Create dummy src and required directories
RUN mkdir -p src static templates i18n
RUN echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release && rm -rf src target/release/deps/frontend*

# Copy all source files and assets
COPY schema.graphql ./
COPY src/ src/
COPY static/ static/
COPY templates/ templates/
COPY i18n/ i18n/
COPY migrations/ migrations/
COPY queries/ queries/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r frontend && useradd --no-log-init -r -g frontend frontend

WORKDIR /app

# Copy the binary and runtime assets
COPY --from=builder /app/target/release/frontend ./
COPY --from=builder /app/templates/ templates/
COPY --from=builder /app/i18n/ i18n/
COPY --from=builder /app/static/ static/

# Set ownership
RUN chown -R frontend:frontend /app

USER frontend

# Environment variables
ENV ENVIRONMENT=production
ENV HOST=0.0.0.0
ENV PORT=8088

EXPOSE 8088

CMD ["./frontend"]