# Plan: Secure Database Webhooks

## Context
We have secured the Rust Orchestrator API (`/ingest/*`) to require a valid JWT. However, the existing PostgreSQL triggers on `public.resumes` and `public.zip_archives` are currently sending unauthenticated requests (or requests with static headers). They need to be updated to dynamically generate and sign a JWT using the project's secret stored in Supabase Vault.

## Strategy
We will implement a "Supabase Native" secure client directly in the database using extensions.
1.  **Vault:** Retrieve the stored `app_jwt_secret`.
2.  **pgjwt:** Dynamically sign a new JWT for each request (short-lived, e.g., 1 hour).
3.  **pg_net:** Send the HTTP POST request with the `Authorization: Bearer <token>` header.

## Implementation Steps

### 1. Enable Extensions
Ensure `pgjwt` is enabled. It is required for the `sign()` function.

```sql
CREATE EXTENSION IF NOT EXISTS pgjwt WITH SCHEMA extensions;
```

### 2. Create Secure Notification Function
Create a PL/pgSQL function `notify_orchestrator_secure` that replaces the generic `http_request` wrapper. This function will:
-   Fetch the secret from `vault.decrypted_secrets`.
-   Mint a token with claims: `role: service_role`, `iss: supabase`, `sub: orchestrator`.
-   Construct the secure headers.
-   Send the payload via `net.http_post`.

**SQL Definition:**
```sql
CREATE OR REPLACE FUNCTION public.notify_orchestrator_secure()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  secret text;
  token text;
  url text := TG_ARGV[0];
  headers jsonb;
  request_id bigint;
  payload jsonb;
  timeout_ms integer := 5000;
BEGIN
  -- 1. Get Secret
  SELECT decrypted_secret INTO secret
  FROM vault.decrypted_secrets
  WHERE name = 'app_jwt_secret';

  IF secret IS NULL THEN
    RAISE EXCEPTION 'app_jwt_secret not found in vault';
  END IF;

  -- 2. Sign Token (valid for 1 hour)
  token := sign(
    json_build_object(
      'role', 'service_role',
      'iss', 'supabase',
      'exp', (extract(epoch from now()) + 3600)::bigint,
      'sub', 'orchestrator'
    ),
    secret
  );

  -- 3. Build Headers
  headers := jsonb_build_object(
    'Content-Type', 'application/json',
    'Authorization', 'Bearer ' || token
  );

  -- 4. Build Payload
  payload := jsonb_build_object(
    'old_record', OLD,
    'record', NEW,
    'type', TG_OP,
    'table', TG_TABLE_NAME,
    'schema', TG_TABLE_SCHEMA
  );

  -- 5. Send Request
  SELECT http_post INTO request_id
  FROM net.http_post(
    url,
    payload,
    '{}'::jsonb, -- params
    headers,
    timeout_ms
  );

  RETURN NEW;
END;
$$;
```

### 3. Update Triggers
Drop the old triggers and create new ones that use the secure function.

**Important:** Confirm the target URL (e.g., ngrok or production URL) before applying.

```sql
-- Drop old triggers
DROP TRIGGER IF EXISTS "orchestrator-ingest-interns-individual" ON public.resumes;
DROP TRIGGER IF EXISTS "orchestrator-injest-resumes-batch" ON public.zip_archives;

-- Create new triggers
CREATE TRIGGER "orchestrator-ingest-interns-individual"
AFTER INSERT ON public.resumes
FOR EACH ROW
EXECUTE FUNCTION public.notify_orchestrator_secure(
  'https://heliochromic-isenthalpic-aidyn.ngrok-free.dev/ingest/interns/individual'
);

CREATE TRIGGER "orchestrator-injest-resumes-batch"
AFTER INSERT ON public.zip_archives
FOR EACH ROW
EXECUTE FUNCTION public.notify_orchestrator_secure(
  'https://heliochromic-isenthalpic-aidyn.ngrok-free.dev/ingest/resumes/batch'
);
```

## Verification
1.  **Start the Rust Server:** Ensure it is running and accessible via the configured URL.
2.  **Upload a File:** Upload a PDF to the `resumes` bucket or a ZIP to `zip-archives`.
3.  **Check Logs:**
    -   **Supabase:** Check `net.http_request_queue` or `postgres` logs for success/failure of the trigger.
    -   **Rust Server:** Verify the request was received and `auth` middleware accepted the token.

## Safety & Rollback

### Atomicity
The migration SQL should always be wrapped in a `BEGIN; ... COMMIT;` block. This ensures that if any part of the migration fails (e.g., function creation fails), the entire operation is rolled back, leaving the database in its original state.

### Revert Script
A dedicated revert script has been prepared at `context/revert_secure_webhooks.sql`.
If the deployment causes issues, run this script to:
1.  Drop the new secure triggers and function.
2.  Restore the original triggers with their static configuration.
