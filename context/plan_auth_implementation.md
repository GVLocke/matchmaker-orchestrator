# Plan: Implement JWT Authentication with Supabase Vault

## Objective
Secure the API endpoints (`/ingest/*`) using JWT authentication. The JWT secret will be stored in and retrieved from Supabase Vault, falling back to an environment variable if not found.

## Prerequisites
-   `supabase_vault` extension is already installed on the project.
-   Project JWT Secret (available in Supabase Dashboard > Project Settings > API).

## Implementation Steps

### 1. Add Dependencies
Add the following to `Cargo.toml`:
-   `jsonwebtoken`: For decoding and verifying tokens.
-   `axum-extra`: For `TypedHeader` and `Authorization` bearer handling.

### 2. Database Setup (Supabase Vault)
The user (you) will need to add the JWT secret to the Vault.
SQL Command:
```sql
-- Replace 'your-project-jwt-secret' with the actual secret
SELECT vault.create_secret('your-project-jwt-secret', 'app_jwt_secret');
```

### 3. Logic Implementation

#### A. Create `src/auth.rs`
-   Define `Claims` struct to deserialize standard Supabase JWTs.
-   Implement a helper function `get_jwt_secret(pool: &PgPool) -> Result<String, Error>`:
    -   Query `vault.decrypted_secrets` for `app_jwt_secret`.
    -   Fallback to `SUPABASE_JWT_SECRET` env var if not in DB.
-   Implement an `auth` middleware function:
    -   Extract `Authorization` header.
    -   Verify token using `jsonwebtoken` and the fetched secret.
    -   Pass control to handler if valid.

#### B. Update `src/main.rs`
-   Load the JWT secret at startup (verify it exists).
-   Store the secret (or the decoding key) in `AppState`.
-   Apply the auth middleware to the router.

### 4. Verification
-   Generate a test JWT using the project secret.
-   Call `/ingest/interns/individual` with the token.
-   Verify 401 Unauthorized without token.
-   Verify 202 Accepted with valid token.

## Action Plan
1.  **Modify `Cargo.toml`** to add dependencies.
2.  **Create `src/auth.rs`** with logic.
3.  **Refactor `src/main.rs`** to integrate auth.
4.  **Create a test script** (or use curl) to verify.
