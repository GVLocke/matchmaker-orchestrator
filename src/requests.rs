mod date_format;
mod supabase;
mod openai;

use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{json, Value, from_str, to_string, from_slice};
use tokio::task;
use supabase::download_pdf;
use openai::generate_structure_from_pdf;

#[derive(Deserialize, Debug)]
pub struct SupabaseWebhook {
    pub r#type: String,
    pub table: String,
    pub record: ResumeRecord,
    pub schema: String,
}

#[derive(Deserialize, Debug)]
pub struct ResumeRecord {
    id: String,
    filename: String,
    created_at: String,
}

pub async fn handle_upload(Json(payload): Json<SupabaseWebhook>) -> impl IntoResponse {
    let filename = payload.record.filename.clone();
    tracing::info!("scrape handler accessed");
    tracing::info!("Upload filename: {}", filename);
    task::spawn(async move {
        let pdf_data = download_pdf(&filename).await.expect("Failed to download pdf");
        let out = pdf_extract::extract_text_from_mem(&pdf_data).unwrap();
        tracing::debug!("PDF Text Contents: {}", out);
        let response = generate_structure_from_pdf(&out).await;
        match response {
            Ok(r) => {
                if let Some(choice) = r.choices.first() {
                    let raw_content = &choice.message.content;
                    let parsed_json : Value = from_str(raw_content).expect("LLM did not return valid JSON!");
                    let pretty_json = to_string(&parsed_json).expect("Failed to format JSON!");
                    tracing::debug!("LLM-generated JSON: {}", pretty_json);
                    tracing::info!("LLM-generated JSON received");
                }
            }
            Err(_) => {
                tracing::error!("{:#?}", response);
            }
        }

    });
    (StatusCode::ACCEPTED, Json(json!({"status": "processing", "message": "We're working on it!"})))
}