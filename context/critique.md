# Codebase Critique: Matchmaker Orchestrator

## Overview
The codebase is currently in a "prototype" state and is not production-ready due to several critical flaws in reliability, scalability, and architecture.

## Critical Issues

### 1. Fragile Error Handling (The "Panic" Problem)
The code is riddled with `.unwrap()` and `.expect()`.
* **Impact:** A single malformed PDF, a temporary network glitch with Supabase, or an unexpected LLM response will cause background tasks to crash silently without any record of the failure.
* **Location:** Heavily present in `src/requests.rs` and `src/main.rs`.

### 2. Invisible Failures
Using `tokio::spawn` for "fire-and-forget" processing without updating a status in the database means that if a task fails, the system (and the user) will never know.
* **Impact:** The HTTP 202 response becomes a "false promise" if the subsequent task dies.

### 3. Resource Exhaustion Risks
* **Unbounded Tasks:** `handle_batch_upload` spawns an unbounded number of tasks for every file in a ZIP archive. This could lead to IP blocking, rate limiting, or memory exhaustion.
* **HTTP Client Anti-pattern:** `reqwest::Client` is instantiated per request in `src/requests/openai.rs` instead of being shared.

### 4. Architectural Bloat
Axum handlers are doing too much. They manage:
* Web/HTTP concerns
* External service orchestration (OpenAI, Supabase)
* File system operations
* Database queries
* **Impact:** High coupling and difficulty in testing. There is no service layer or separation of concerns.

### 5. Configuration and Scalability
* **Hardcoded Values:** Secrets and configurations (like the OpenAI model) are fetched from the environment repeatedly or hardcoded.
* **Placeholder Logic:** The use of `gpt-5-nano` indicates a lack of verification with actual available models.

## Strategic Recommendations

1. **Job Status Tracking:** Implement a "Job" or "Task" status table in the database to track background processing states (Pending, Processing, Completed, Failed).
2. **Robust Error Handling:** Replace all `unwrap`/`expect` with proper error propagation using the `?` operator and a crate like `thiserror` or `anyhow`.
3. **Service Layer Refactoring:** Move business logic into a `Service` struct that can be injected into Axum handlers via `State`.
4. **Shared State:** Use a single `reqwest::Client` stored in Axum's `State`.
5. **Concurrency Control:** Implement a `Semaphore` to limit the number of concurrent PDF processing tasks.
6. **Testing:** Add unit tests for PDF parsing and LLM response mapping logic.
