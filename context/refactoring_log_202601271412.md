# Refactoring Log - 2026-01-27

## Changes Summary

### 1. Storage Infrastructure Migration
- **Substituted** `supabase-lib-rs` with the official `aws-sdk-s3` crate.
- **Reasoning**: Decoupled from unofficial libraries and aligned with industry standards for S3-compatible storage.
- **Implementation**:
    - Updated `AppState` to hold `aws_sdk_s3::Client`.
    - Implemented a robust endpoint resolution system that detects local vs. remote Supabase instances.
    - Added support for `S3_ACCESS_KEY` and `S3_SECRET_KEY` in `.env` to avoid hardcoding credentials.
    - Configured `force_path_style(true)` to satisfy Supabase Storage requirements.

### 2. Job Status Tracking
- **Schema Updates**:
    - Created `job_status` ENUM (`pending`, `processing`, `completed`, `failed`).
    - Added `status` and `error_message` columns to `resumes` and `zip_archives`.
- **Logic**:
    - `ResumeService` now updates record status at each stage of the lifecycle.
    - Error messages from S3 and PDF processing are now captured in the database for easier debugging.

### 3. Data Lineage (Batch Processing)
- **Problem**: Individual resumes extracted from a ZIP were not linked to their parent archive.
- **Solution**: 
    - Added `zip_id` foreign key to the `resumes` table.
    - Implemented explicit linking in `ResumeService` after S3 upload.
    - Includes a retry mechanism to handle potential race conditions with database triggers.

## Technical Notes
- **Local Dev Endpoint**: For local Supabase (127.0.0.1:54321), the S3 endpoint must be constructed as `http://127.0.0.1:54321/storage/v1/s3` with the correct Access Key ID (e.g., from `supabase status`).
- **Trigger Alignment**: Database triggers (`handle_new_resume_upload`) are synchronized with the orchestrator via standard Postgres notifications or direct HTTP calls.