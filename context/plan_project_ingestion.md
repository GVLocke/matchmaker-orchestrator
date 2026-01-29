# Plan: Project Data Ingestion

## 1. Database Schema
Create infrastructure to track uploads and store project details.

- **`project_uploads` Table**: Tracks the status of spreadsheet files.
    - `id` (UUID, PK)
    - `filename` (TEXT)
    - `status` (job_status: `pending`, `processing`, `completed`, `failed`)
    - `error_message` (TEXT, optional)
    - `created_at` (TIMESTAMPTZ)
- **`projects` Table**: Stores individual project records.
    - `id` (UUID, PK)
    - `upload_id` (UUID, FK -> project_uploads.id)
    - `title` (TEXT)
    - `description` (TEXT)
    - `requirements` (TEXT)
    - `manager` (TEXT)
    - `deadline` (TEXT)
    - `priority` (SMALLINT)
    - `intern_cap` (SMALLINT) (project's capacity for interns)
    - `created_at` (TIMESTAMPTZ)

## 2. Storage Setup
- Create a new Supabase Storage bucket named `project-spreadsheets`.
- Configure bucket-level triggers (similar to resumes) to ensure database records are created when files are uploaded or deleted.

## 3. Backend Implementation (Rust)
- **Dependencies**: Add `csv` and `calamine` (for Excel) to `Cargo.toml`.
- **New Route**: `POST /ingest/projects`
- **Service Logic**:
    - Download file from `project-spreadsheets` bucket.
    - Identify format by extension (`.csv` vs `.xlsx`).
    - Parse rows using a header-mapping strategy (e.g., matching "Title", "Project Name", etc.).
    - Perform a batch `INSERT` into the `projects` table.
    - Update `project_uploads` status.

## 4. Future Considerations
- **Vector Embeddings**: Add `vector` columns to `projects` and `resumes` once the AI model is selected.
- **Data Validation**: Strict typing for deadlines and priorities.
- **Deduplication**: Logic to handle re-uploads of the same project list.
