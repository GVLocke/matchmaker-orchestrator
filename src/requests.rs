mod date_format;
mod supabase;
mod openai;

use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, from_str, to_string_pretty};
use tokio::task;
use supabase::download_pdf;
use openai::generate_structure_from_pdf;
use crate::requests::supabase::insert_into_resume_table;

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadRequest {
    filename: String
}

pub async fn handle_upload(Json(payload): Json<UploadRequest>) -> impl IntoResponse {
    let filename = payload.filename.clone();

    task::spawn(async move {
        let pdf_data = download_pdf(&filename).await.unwrap();
        let out = pdf_extract::extract_text_from_mem(&pdf_data).unwrap();
        println!("{}", out);
        let response = generate_structure_from_pdf(&out).await;
        match response {
            Ok(r) => {
                if let Some(choice) = r.choices.first() {
                    let raw_content = &choice.message.content;
                    let parsed_json : Value = from_str(raw_content).expect("LLM did not return valid JSON!");
                    let pretty_json = to_string_pretty(&parsed_json).expect("Failed to format JSON!");
                    println!("{}", pretty_json);
                    let resume = insert_into_resume_table(&filename, parsed_json, &out).await.expect("Failed to insert into table!");
                    println!("{:#?}", resume);
                }
            }
            Err(_) => {
                println!("{:#?}", response);
            }
        }

    });

    (StatusCode::ACCEPTED, Json(json!({"status": "processing", "message": "We're working on it!"})))
}