# Changelog - 2026-01-28

## Cloud Migration & Infrastructure
- **Supabase Cloud Link**: Linked local workspace to the `InternProjectMatchmaker` project.
- **Environment Configuration**: Updated `.env` to use the Supabase Session Pooler (IPv4 support) and cloud S3 credentials.
- **Remote Schema Sync**: 
    - Created and applied migrations to align the cloud database with the required orchestrator schema.
    - Added `job_status` enum and `zip_archives` table.
    - Added `status`, `error_message`, and `zip_id` columns to the `resumes` table.
    - Synced the `results_tab` table from cloud to local migration history.

## Storage & Automation
- **Storage Triggers**: Implemented PostgreSQL triggers on `storage.objects` to automatically link file uploads/deletions in the `resumes` and `zip-archives` buckets to their respective database tables.
- **Webhook Integration**: Verified that cloud webhooks successfully reach the local orchestrator via **ngrok** tunnels.

## Verification
- **End-to-End Workflow**: Confirmed that uploading a ZIP file to the cloud bucket correctly triggers the extraction process, re-uploads individual PDFs, and initiates resume parsing.
- **Database Status Tracking**: Verified that resume and ZIP records correctly transition through `processing`, `completed`, and `failed` states with error logging.
