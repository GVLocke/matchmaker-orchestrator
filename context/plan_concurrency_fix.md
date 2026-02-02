# Plan: Concurrency Fix via Metadata-Driven Triggers

## Objective
Eliminate the race condition in `handle_batch_extraction` where the Rust service attempts to link extracted resumes to a zip file before the database row exists.

## Strategy
Switch from an "Upload then Update" pattern to a "Upload with Metadata" pattern.
1.  **Rust**: When re-uploading extracted PDFs, attach the `zip_id` as S3 Metadata (`x-amz-meta-zip_id`).
2.  **Postgres**: Modify the `storage.objects` trigger to read this metadata and insert it directly into the `resumes` table upon creation.

## Implementation Steps

### 1. Database Migration (PostgreSQL)
We need to update the function that handles new inserts into the `resumes` table.
*   **Identify Function**: Find the function currently triggered by uploads to `resumes`.
*   **Modify Logic**:
    *   Extract `zip_id` from `new.metadata` (JSONB).
    *   Insert into `public.resumes` with `(filename, status, zip_id)`.
    *   Handle `ON CONFLICT` if necessary (though strictly speaking, this should be the *only* creation path).

**SQL Logic Draft:**
```sql
CREATE OR REPLACE FUNCTION public.handle_new_resume()
RETURNS trigger
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
  zip_id_val uuid;
BEGIN
  -- Extract zip_id from metadata if it exists
  -- Note: metadata keys are often lowercase in Supabase Storage
  zip_id_val := (new.metadata ->> 'zip_id')::uuid;

  INSERT INTO public.resumes (id, filename, status, zip_id)
  VALUES (
    gen_random_uuid(), -- or new.id if we are mirroring IDs
    new.name,
    'pending',
    zip_id_val
  );

  RETURN new;
END;
$$;
```

### 2. Rust Service Update (`src/service.rs`)
Update `handle_batch_extraction`:
*   Remove the `while retries < 3` loop and the `UPDATE resumes SET zip_id ...` query.
*   Update `s3_client.put_object()` call to include `.metadata("zip_id", id.to_string())`.

### 3. Verification
*   **Clean Test**: Upload a ZIP file.
*   **Check**:
    1.  Does the `resumes` table populate immediately?
    2.  Do the new rows have the correct `zip_id`?
    3.  Does the processing pipeline continue successfully?

## Rollback Plan
*   Revert Rust code to use the retry loop.
*   Revert the SQL function to the previous version (ignoring metadata).
