use axum::{
    body::Body,
    extract::{State, Request},
    http::{StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: Option<String>,
    pub exp: usize,
    pub sub: String,
    pub role: Option<String>,
}

pub async fn get_jwt_secret(pool: &PgPool) -> anyhow::Result<String> {
    // Try to get from Vault first
    let row = sqlx::query!(
        "SELECT decrypted_secret FROM vault.decrypted_secrets WHERE name = 'app_jwt_secret'"
    )
    .fetch_optional(pool)
    .await?;

    if let Some(record) = row {
        if let Some(s) = record.decrypted_secret {
            return Ok(s);
        }
    }

    // Fallback
    env::var("SUPABASE_JWT_SECRET").map_err(|_| anyhow::anyhow!("JWT Secret not found"))
}

pub async fn auth(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .typed_try_get::<Authorization<Bearer>>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?
        .token()
        .to_string();

    let secret = &state.jwt_secret;
    let validation = Validation::new(Algorithm::HS256);
    
    let _ = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(req).await)
}
