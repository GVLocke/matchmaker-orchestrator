# Project TODOs

- [x] **Database Access Strategy**: Re-evaluate direct SQL vs. PostgREST for production.
    - *Context*: Concern that Cloud Supabase Postgres instance is harder to access via direct SQL in production.
    - *Decision*: Stick with `sqlx` (Direct SQL) for type safety and performance in the Axum backend.
- [x] **Migrate Storage to standard S3 crate**: Replace `supabase-lib-rs` with `aws-sdk-s3`.
    - *Context*: Decouple from unofficial Supabase crates and use standard S3 protocols for better reliability and features.
    - *Result*: Successfully migrated to `aws-sdk-s3` with support for local and remote endpoints via `.env`.
- [x] **Add Status Tracking**: Implement state tracking for long-running jobs.
    - *Tasks*:
        - [x] Add `status` column (e.g., `pending`, `processing`, `completed`, `failed`) to `resumes` and `zip_archives` tables.
        - [x] **Add `zip_id` foreign key** to `resumes` table (nullable) to track lineage from `zip_archives`.
        - [x] Update `ResumeService` to report status transitions.
        - [x] Expose status via API (accessible via direct DB query/Supabase).
- [x] **Batch and Single Upload Endpoints**: Ensure endpoints exist for single and batch file uploads.
    - *Context*: `POST /scrape/individual` and `POST /scrape/batch` are already implemented in `src/requests.rs` and registered in `src/main.rs`.

## New Features (from Project Scope)
- [ ] **Project Data Ingestion**: Parse project spreadsheets (CSV/XLSX) and insert into DB.
- [ ] **Vector Embeddings**: Generate embeddings for structured Resume JSON and Project data.
- [ ] **Neural Network Integration**: Create interface to query the custom neural network with embeddings.

## Quality & Infrastructure
- [ ] **Unit Testing**: Implement tests for core logic.
    - *Focus*: `ResumeService` PDF processing, LLM response parsing, and database interaction mocks.
- [ ] **Verify Batch Trigger**: Ensure uploading extracted PDFs from a batch actually triggers the `scrape/individual` workflow (via Supabase Storage Webhooks or DB triggers).



