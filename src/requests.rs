mod openai;

use ::supabase::Client;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use serde::{Deserialize};
use serde_json::{json, Value, from_str, to_string};
use sqlx::{Error, PgPool};
use sqlx::postgres::PgQueryResult;
use tokio::task;
use openai::generate_structure_from_pdf;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct SupabaseWebhook {
    // pub r#type: String,
    // pub table: String,
    pub record: ResumeRecord,
    // pub schema: String,
}

#[derive(Deserialize, Debug)]
pub struct ResumeRecord {
    id: Uuid,
    filename: String,
}

pub async fn handle_upload(State(pool): State<PgPool>, Extension(client): Extension<Client>, Json(payload): Json<SupabaseWebhook>) -> impl IntoResponse {
    let filename = payload.record.filename.clone();
    let id = payload.record.id.clone();
    tracing::info!("scrape handler accessed");
    tracing::info!("Upload filename: {}; Id: {}", filename, id);
    task::spawn(async move {
        let pdf_data = client.storage().download("resumes", &filename).await.expect("Failed to download pdf");
        let out = pdf_extract::extract_text_from_mem(&pdf_data).unwrap();
        tracing::debug!("PDF Text Contents for filename {}, id {}: {}", filename, id, out);
        let response = generate_structure_from_pdf(&out).await;
        match response {
            Ok(r) => {
                if let Some(choice) = r.choices.first() {
                    let raw_content = &choice.message.content;
                    let parsed_json : Value = from_str(raw_content).expect("LLM did not return valid JSON!");
                    let pretty_json = to_string(&parsed_json).expect("Failed to format JSON!");
                    tracing::info!("LLM-generated JSON received for filename {}, id {}", filename, id);
                    tracing::debug!("LLM-generated JSON for filename {}, id {}: {}", filename, id, pretty_json);
                    match update_resume_record(&pool, id, out, parsed_json).await {
                        Ok(_) => {
                            tracing::info!("Resume record {} (filename: {}) updated successfully", id, filename);
                        }
                        Err(_) => {
                            tracing::error!("Failed to update resume record for filename {}, id {}", filename, id);
                        }
                    }
                }
            }
            Err(_) => {
                tracing::error!("{:#?}", response);
            }
        }
    });
    (StatusCode::ACCEPTED, Json(json!({"status": "processing", "message": "We're working on it!"})))
}

pub async fn update_resume_record(pool: &PgPool, id: Uuid, text: String, structured_json : Value) -> Result<PgQueryResult, Error> {
    sqlx::query!("UPDATE resumes SET text = $1, structured = $2 WHERE id = $3", text, structured_json, id).execute(pool).await
}