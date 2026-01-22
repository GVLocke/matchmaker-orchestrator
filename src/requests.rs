mod openai;

use std::io::{Read, Write};
use ::supabase::Client;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum::body::Bytes;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{json, Value, from_str, to_string};
use sqlx::{Error, PgPool};
use sqlx::postgres::PgQueryResult;
use tempfile::tempfile;
use tokio::task;
use openai::generate_structure_from_pdf;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct WebhookPayload {
    pub record: FileTrackingTableRecord,
}

#[derive(Deserialize, Debug)]
pub struct FileTrackingTableRecord {
    id: Uuid,
    filename: String,
}

pub async fn handle_single_upload(State(pool): State<PgPool>, Extension(client): Extension<Client>, Json(payload): Json<WebhookPayload>) -> impl IntoResponse {
    let filename = payload.record.filename.clone();
    let id = payload.record.id.clone();
    tracing::info!("scrape handler accessed");
    tracing::info!("Upload filename: {}; Id: {}", filename, id);
    task::spawn(async move {
        let pdf_data : Bytes = client.storage().download("resumes", &filename).await.expect("Failed to download pdf");
        match process_single_pdf(&pdf_data, &*filename, id).await {
            Some((pdf_text, parsed_json)) => {
                match update_resume_record_from_individual_upload(&pool, id, pdf_text, parsed_json).await {
                    Ok(_) => {
                        tracing::info!("Resume record {} (filename: {}) updated successfully", id, filename);
                    }
                    Err(_) => {
                        tracing::error!("Failed to update resume record for filename {}, id {}", filename, id);
                    }
                }
            }
            None => {}
        };
    });
    (StatusCode::ACCEPTED, Json(json!({"status": "processing", "message": "We're working on it!"})))
}

pub async fn handle_batch_upload(
    Extension(client): Extension<Client>,
    Json(payload): Json<WebhookPayload>
) -> impl IntoResponse {
    let filename = payload.record.filename.clone();
    let id = payload.record.id.clone();
    tracing::info!("batch upload handler accessed");
    tracing::info!("Upload filename: {}; Id: {}", filename, id);
    task::spawn(async move {
        let zip_data = client.storage()
            .download("zip-archives", &filename)
            .await.expect("Failed to download zip");
        let mut tmp_file = tempfile().expect("Failed to create tempfile");
        tmp_file.write_all(&zip_data).expect("Failed to write to tempfile");
        let mut archive = zip::ZipArchive::new(tmp_file).expect("Failed to create zip archive");
        tracing::info!("Successfully extracted zip archive");
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            tracing::info!("Filename: {}", file.name());

            if file.is_dir() || !file.name().ends_with(".pdf") {
                continue;
            }

            tracing::info!("Processing file inside zip: {}", file.name());
            let mut pdf_buffer = Vec::new();
            file.read_to_end(&mut pdf_buffer).expect("Failed to read file to buffer");
            let pdf_bytes = Bytes::from(pdf_buffer);
            let pdf_name = file.name().to_string();

            let client = client.clone();
            let upload_path = format!("{}_{}", filename, pdf_name);
            tracing::info!("upload path: {}", upload_path);

            tokio::spawn(async move {
                let options = supabase::storage::FileOptions {
                    cache_control: None,
                    content_type: Some("application/pdf".to_string()),
                    upsert: false,
                };

                client.storage()
                    .upload("resumes", &upload_path, pdf_bytes, Some(options))
                    .await.expect("Failed to upload resumes");
            });
        }
    });
    (StatusCode::ACCEPTED, Json(json!({"status": "processing", "message": "We're working on it!"})))
}

pub async fn process_single_pdf(
    pdf_data: &Bytes,
    filename: &str,
    id: Uuid
) -> Option<(String, Value)>{
    let pdf_text = pdf_extract::extract_text_from_mem(pdf_data).unwrap();
    tracing::debug!("PDF Text Contents for filename {}, id {}: {}", filename, id, pdf_text);
    let response = generate_structure_from_pdf(&pdf_text).await;
    match response {
        Ok(r) => {
            if let Some(choice) = r.choices.first() {
                let raw_content = &choice.message.content;
                let parsed_json : Value = from_str(raw_content).expect("LLM did not return valid JSON!");
                let pretty_json = to_string(&parsed_json).expect("Failed to format JSON!");
                tracing::info!("LLM-generated JSON received for filename {}, id {}", filename, id);
                tracing::debug!("LLM-generated JSON for filename {}, id {}: {}", filename, id, pretty_json);
                Option::from((pdf_text, parsed_json))
            }
            else {
                tracing::error!("Failed to parse LLM-generated JSON for filename {}, id {}", filename, id);
                None
            }
        }
        Err(_) => {
            tracing::error!("{:#?}", response);
            None
        }
    }
}

pub async fn update_resume_record_from_individual_upload(pool: &PgPool, id: Uuid, text: String, structured_json : Value) -> Result<PgQueryResult, Error> {
    sqlx::query!("UPDATE resumes SET text = $1, structured = $2 WHERE id = $3", text, structured_json, id).execute(pool).await
}