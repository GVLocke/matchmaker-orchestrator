# Plan: Term Tracking Implementation

## Objective
Enable tracking of the "Term" (e.g., "Spring 2026", "Summer 2025") for all resumes and projects. The term will be automatically extracted from the file path in Supabase Storage.

## 1. Database Schema Changes
We need to add the `term` column to our core tables and update the ingestion triggers.

### A. Table Updates
Add a `term` column (Nullable TEXT) to:
- `resumes`
- `zip_archives`
- `project_uploads`
- `projects`

### B. Trigger Function Updates
Update the following functions to extract the term from the storage object's path:
- `handle_new_resume_upload()`
- `handle_new_zip_upload()`
- `handle_new_project_upload()`

**Logic for extraction:**
If the filename contains a `/`, take the first segment as the term. Otherwise, set it to `NULL`.

### C. Projects Table Logic
When projects are inserted into the `projects` table by the Rust service, they should inherit the `term` from their parent `project_uploads` record.

## 2. Backend Implementation (Rust)
Update the `ProjectService` to propagate the term.

### A. `ProjectService::insert_projects`
1.  Fetch the `term` from the `project_uploads` table using the `upload_id`.
2.  Include this `term` in the `INSERT` query for each project.

### B. Concurrency Considerations
When `handle_batch_extraction` re-uploads PDFs to the `resumes` bucket, it should preserve the path. It almost does this, currently, except it modifies the filename while keeping the file at the root of the bucket. This needs to be modified so that the resumes go into a subdirectory named by the term. Once this is implemented, the database trigger on the `resumes` bucket will then naturally extract the term for the individual resume records.

## 3. Verification Steps

1.  **Direct Upload**: Upload a file to `Spring 2026/resume.pdf` in the `resumes` bucket and verify the `term` column is set to "Spring 2026".
2.  **Batch Upload**: Upload a ZIP to `Summer 2025/batch.zip` in the `zip-archives` bucket. Verify:
    - The `zip_archives` record has `term = 'Summer 2025'`.
    - Extracted resumes in the `resumes` table also have `term = 'Summer 2025'`.
3.  **Project Upload**: Upload a spreadsheet to `Fall 2026/projects.xlsx`. Verify:
    - The `project_uploads` record has `term = 'Fall 2026'`.
    - All rows in the `projects` table linked to this upload have `term = 'Fall 2026'`.

## Rollback Plan
- SQL migration to drop the `term` columns and revert trigger functions.
