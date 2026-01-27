# Matchmaker Orchestrator

## Project Overview
This is a Rust-based backend service built with **Axum** designed to orchestrate the processing of resume files. It acts as a middleware between Supabase Storage, an OpenAI LLM, and a PostgreSQL database.

**Core Workflow:**
1.  Receives HTTP webhooks (single file or batch ZIP).
2.  Downloads files (PDFs or ZIPs) from **Supabase Storage**.
3.  Extracts raw text from PDF files using `pdf-extract`.
4.  Sends the raw text to **OpenAI** to parse it into a structured JSON format (Education, Skills, Experience).
5.  Updates the corresponding record in a **PostgreSQL** database (via `sqlx`).

## Architecture
*   **Framework:** Axum (Web Server)
*   **Runtime:** Tokio (Async)
*   **Database:** PostgreSQL (via `sqlx`)
*   **Storage:** Supabase Storage
*   **AI Integration:** OpenAI API (for unstructured text parsing)
*   **Logging:** Tracing & Tracing Subscriber
*   **Error Handling:** Uses `anyhow` for error propagation and `tracing` for structured logging. Panics are avoided in background tasks.

## Key Files
*   `src/main.rs`: Entry point. Initializes `AppState` (shared DB pool, HTTP client, configuration, Semaphore) and sets up Axum routes.
*   `src/service.rs`: **Core Business Logic.** Contains the `ResumeService` struct which handles PDF extraction, LLM orchestration, ZIP processing, and DB updates.
*   `src/requests.rs`: HTTP Route handlers (`handle_single_upload`, `handle_batch_upload`). These are lightweight wrappers that delegate to `ResumeService`.
*   `src/requests/openai.rs`: Helper functions for communicating with the OpenAI API.
*   `src/resume_schema.json`: Defines the expected JSON structure for the parsed resume data.
*   `Cargo.toml`: Project dependencies.

## Building and Running

### Prerequisites
*   Rust (latest stable)
*   PostgreSQL database (or Supabase project)
*   OpenAI API Key

### Environment Variables
The application requires a `.env` file or system environment variables:
```env
DATABASE_URL=postgres://user:pass@host:port/dbname
SUPABASE_ENDPOINT=https://your-project.supabase.co
SERVICE_KEY=your-supabase-service-role-key
OPENAI_API_KEY=your-openai-api-key
MAX_CONCURRENT_TASKS=10  # Optional: Defaults to 10 if not set
```

### Commands
*   **Run Development Server:**
    ```bash
    cargo run
    ```
    The server listens on `0.0.0.0:3000`.

*   **Build for Production:**
    ```bash
    cargo build --release
    ```

## API Endpoints

### `POST /scrape/individual`
Triggers processing for a single uploaded PDF.
*   **Payload:** JSON containing the file record (ID and filename).
*   **Behavior:** Spawns a background task via `ResumeService`. Returns HTTP 202 Accepted immediately.

### `POST /scrape/batch`
Triggers processing for a ZIP archive of resumes.
*   **Payload:** JSON containing the file record.
*   **Behavior:** Spawns a background task via `ResumeService` to extract the ZIP and re-upload individual PDFs. Returns HTTP 202 Accepted.

## Development Conventions
*   **State Management:** All shared state is held in `AppState` and injected via Axum's `State` extractor.
*   **Concurrency:** Uses `tokio::spawn` for background tasks, throttled by a `tokio::sync::Semaphore` (limit defined by `MAX_CONCURRENT_TASKS`) to prevent resource exhaustion.
*   **Database:** Uses `sqlx` with compile-time checked queries (mostly).
*   **Logging:** Uses structured logging via `tracing`. Failures in background tasks are logged as errors.