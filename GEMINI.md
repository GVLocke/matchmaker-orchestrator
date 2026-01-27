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

## Key Files
*   `src/main.rs`: Application entry point. Configures environment variables, database connection pool, Supabase client, and HTTP routes.
*   `src/requests.rs`: Contains the route handlers (`handle_single_upload`, `handle_batch_upload`) and the core business logic for file processing and LLM interaction.
*   `src/requests/openai.rs`: Handles communication with the OpenAI API, including schema generation.
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
*   **Behavior:** Downloads PDF, extracts text, structures it via LLM, updates DB. Returns HTTP 202 Accepted immediately; processing happens in the background.

### `POST /scrape/batch`
Triggers processing for a ZIP archive of resumes.
*   **Payload:** JSON containing the file record.
*   **Behavior:** Downloads ZIP, extracts contents, filters for PDFs, re-uploads individual PDFs to storage, and spawns processing tasks for each. Returns HTTP 202 Accepted.

## Development Conventions
*   **Error Handling:** Currently relies heavily on panics (`.unwrap()`, `.expect()`) within background tasks. *Note: This is an active area for refactoring.*
*   **Concurrency:** Uses `tokio::spawn` for fire-and-forget background processing.
*   **Database:** Uses `sqlx` with compile-time checked queries (mostly).
*   **Logging:** Uses structured logging via `tracing`.
