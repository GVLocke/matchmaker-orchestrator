-- Revert script for "Secure Database Webhooks"
-- Usage: Run this to restore the database to its state before the secure webhooks migration.

BEGIN;

-- 1. Drop the new secure triggers
DROP TRIGGER IF EXISTS "orchestrator-ingest-interns-individual" ON public.resumes;
DROP TRIGGER IF EXISTS "orchestrator-injest-resumes-batch" ON public.zip_archives;

-- 2. Drop the secure function
DROP FUNCTION IF EXISTS public.notify_orchestrator_secure();

-- 2b. Drop debug artifacts if they exist
DROP TABLE IF EXISTS public.debug_tokens;

-- 3. Restore the old "insecure" triggers (using the original static headers)
-- Note: Preserving the original ngrok URL. Update if necessary.

CREATE TRIGGER "orchestrator-ingest-interns-individual"
AFTER INSERT ON public.resumes
FOR EACH ROW
EXECUTE FUNCTION supabase_functions.http_request(
    'https://heliochromic-isenthalpic-aidyn.ngrok-free.dev/ingest/interns/individual',
    'POST',
    '{"Content-type":"application/json"}',
    '{}',
    '5000'
);

CREATE TRIGGER "orchestrator-injest-resumes-batch"
AFTER INSERT ON public.zip_archives
FOR EACH ROW
EXECUTE FUNCTION supabase_functions.http_request(
    'https://heliochromic-isenthalpic-aidyn.ngrok-free.dev/ingest/resumes/batch',
    'POST',
    '{"Content-type":"application/json"}',
    '{}',
    '5000'
);

-- 4. (Optional) Disable pgjwt if it wasn't used elsewhere
-- DROP EXTENSION IF EXISTS pgjwt;

COMMIT;
