# Refactoring Log: Stability and Architecture Improvements

**Date:** January 27, 2026
**Focus:** Reliability, Concurrency Control, and Architectural Decoupling

## Summary
This refactoring session addressed critical stability and architectural issues identified in the codebase critique. The primary goals were to eliminate silent failures ("panics"), prevent resource exhaustion during batch processing, and decouple business logic from HTTP handlers.

## Detailed Changes

### 1. Robust Error Handling (The "Panic" Problem)
*   **Issue:** The original code relied heavily on `.unwrap()` and `.expect()` in background tasks. A single failure (e.g., a network glitch or malformed PDF) would cause the task to crash silently, leaving no trace in the logs.
*   **Resolution:** 
    *   Replaced dangerous unwraps with `Result` based error propagation.
    *   Introduced `anyhow` for simplified error context handling.
    *   Implemented comprehensive logging using `tracing::error!` and `tracing::warn!` to capture and report failures in background tasks without crashing the application.

### 2. Shared Infrastructure & Resource Management
*   **Issue:** `reqwest::Client` was being instantiated for every single OpenAI request, and configuration values (like API keys and schemas) were being read from the environment repeatedly.
*   **Resolution:**
    *   Introduced a global `AppState` struct to hold shared resources.
    *   **Shared HTTP Client:** Created a single `reqwest::Client` in `main.rs` and shared it via `AppState`. This allows for connection pooling and better performance.
    *   **Pre-loaded Configuration:** The OpenAI API key and the Resume JSON Schema are now loaded and parsed *once* at startup and stored in `AppState`.

### 3. Concurrency Control
*   **Issue:** The `handle_batch_upload` function would spawn an unbounded number of tasks for every file in a ZIP archive. This posed a severe risk of IP blocking, rate limiting (429 errors), and memory exhaustion.
*   **Resolution:**
    *   Added a `tokio::sync::Semaphore` to `AppState`.
    *   Configured the limit via a `MAX_CONCURRENT_TASKS` environment variable (defaulting to 10).
    *   Both individual PDF processing and batch re-upload tasks now acquire a permit from this semaphore before proceeding, effectively throttling the workload to safe levels.

### 4. Service Layer Architecture
*   **Issue:** Axum handlers in `src/requests.rs` were monolithic, mixing HTTP concerns with file processing, database logic, and external API calls.
*   **Resolution:**
    *   Created `src/service.rs`.
    *   Extracted all business logic (PDF extraction, LLM orchestration, Database updates, ZIP handling) into a new `ResumeService` struct.
    *   Axum handlers now act as a thin layer that simply instantiates the service and delegates the task.

### 5. Dependency Cleanup
*   **Removed:** Unused `Extension` wrapper usage in favor of Axum's type-safe `State`.
*   **Added:** `anyhow` for error handling.

## Next Steps
*   **Database Status Tracking:** While logging is improved, the database still lacks a persistent record of job status (e.g., "Pending", "Failed"). A schema migration to add a `status` column to the `resumes` table is recommended.
*   **Testing:** With the logic now in `ResumeService`, writing unit and integration tests is significantly easier.
