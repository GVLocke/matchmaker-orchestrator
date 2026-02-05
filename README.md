# Matchmaker Orchestrator

A Rust-based backend service built with **Axum** designed to orchestrate the processing of resume files. It acts as a middleware between Supabase Storage (via S3 protocol), an OpenAI LLM, and a PostgreSQL database.

## Core Workflow

1.  **Receive Webhooks:** Receives HTTP webhooks for single file uploads or batch ZIP archives.
2.  **Download:** Downloads files (PDFs or ZIPs) from **Supabase Storage** using the **AWS S3 SDK**.
3.  **Extract:** Extracts raw text from PDF files using `pdf-extract`.
4.  **Analyze:** Sends raw text to **OpenAI** to parse into a structured JSON format based on a predefined schema.
5.  **Persist:** Updates the corresponding record in **PostgreSQL** (via `sqlx`), tracking status (`pending`, `processing`, `completed`, `failed`) and lineage (linking resumes to their parent ZIP).

## Tech Stack

*   **Language:** Rust (Edition 2024)
*   **Web Framework:** [Axum](https://github.com/tokio-rs/axum)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Database:** PostgreSQL with [sqlx](https://github.com/launchbadge/sqlx)
*   **Storage:** Supabase Storage (S3-compatible via `aws-sdk-s3`)
*   **AI:** OpenAI API for resume parsing
*   **Auth:** JWT validation for secure webhooks

## Project Structure

*   `src/main.rs`: Entry point. Initializes application state and sets up routes.
*   `src/service.rs`: **Core Business Logic.** Handles PDF extraction, LLM orchestration, ZIP processing, and DB updates.
*   `src/requests.rs`: HTTP route handlers.
*   `src/auth.rs`: JWT authentication middleware for protecting endpoints.
*   `src/requests/openai.rs`: OpenAI API integration helpers.
*   `src/resume_schema.json`: JSON schema for the parsed resume data.

## Getting Started

### Prerequisites

*   Rust (latest stable)
*   PostgreSQL database (or Supabase project)
*   OpenAI API Key

### Configuration

Create a `.env` file in the root directory:

```env
DATABASE_URL=postgres://postgres.[PROJ_REF]:[PASS]@aws-1-us-east-2.pooler.supabase.com:5432/postgres
SUPABASE_ENDPOINT=https://your-project.supabase.co
SERVICE_KEY=your-supabase-service-role-key
OPENAI_API_KEY=your-openai-api-key
S3_ACCESS_KEY=your-supabase-s3-access-key
S3_SECRET_KEY=your-supabase-s3-secret-key
SUPABASE_JWT_SECRET=your-jwt-secret
MAX_CONCURRENT_TASKS=10
```

### Running the Application

```bash
# Run development server
cargo run

# Build for production
cargo build --release
```

The server listens on `0.0.0.0:3000` by default.

## API Endpoints

### `POST /scrape/individual`
Processes a single uploaded PDF.
*   **Payload:** JSON with file ID and filename.
*   **Response:** `202 Accepted` (processing continues in background).

### `POST /scrape/batch`
Processes a ZIP archive containing multiple resumes.
*   **Payload:** JSON with file ID and filename.
*   **Response:** `202 Accepted`.

## Development

### Concurrency
The application uses `tokio::spawn` for background tasks, throttled by a `tokio::sync::Semaphore` to prevent resource exhaustion. The limit is configurable via `MAX_CONCURRENT_TASKS`.

### Database
Queries are managed with `sqlx`, ensuring compile-time safety for most database interactions.

### Logging
Structured logging is implemented via `tracing` and `tracing-subscriber`.
