# Changelog - 2026-02-05

## Features & Improvements
- **Term Tracking**:
    - Added `term` column to `resumes`, `zip_archives`, `project_uploads`, and `projects` tables.
    - Updated storage triggers to automatically extract term metadata from file paths (e.g., `Spring 2026/file.pdf`).
    - Propagated term data from spreadsheet uploads to individual project records.
- **Robust Storage Ingestion**:
    - Updated storage triggers to ignore `.emptyFolderPlaceholder` files created by the Supabase Dashboard.
    - Added filtering in `ResumeService` to skip hidden/system files during ZIP extraction.
- **Documentation**:
    - Generated a comprehensive `README.md` covering architecture, setup, and API usage.
    - Updated `GEMINI.md` to reflect new project capabilities and configurations.

## API & Infrastructure
- **Endpoint Alignment**: Standardized batch ingestion routes under the `/ingest/interns/batch` path.
- **Security**: Switched repository visibility to **Public** on GitHub.
- **Git Cleanup**: Optimized `.gitignore` and removed tracked IDE metadata (`.idea/`) from the repository index.
