use std::io::{Read, Write};
use axum::body::Bytes;
use serde_json::{Value, from_str, to_string};
use tempfile::tempfile;
use uuid::Uuid;
use crate::AppState;
use crate::requests::openai::generate_structure_from_pdf;

pub struct ResumeService {
    state: AppState,
}

impl ResumeService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn process_and_update_resume(&self, id: Uuid, filename: String) {
        let _permit = self.state.semaphore.acquire().await.expect("Semaphore closed");
        
        // Download
        let pdf_data = match self.state.supabase.storage().download("resumes", &filename).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to download pdf for filename {}, id {}: {}", filename, id, e);
                return;
            }
        };

        // Parse and process
        match self.process_single_pdf(&pdf_data, &filename, id).await {
            Some((pdf_text, parsed_json)) => {
                match self.update_resume_record(id, pdf_text, parsed_json).await {
                    Ok(_) => {
                        tracing::info!("Resume record {} (filename: {}) updated successfully", id, filename);
                    }
                    Err(e) => {
                        tracing::error!("Failed to update resume record for filename {}, id {}: {}", filename, id, e);
                    }
                }
            }
            None => {
                 tracing::warn!("Processing returned None for filename {}, id {}", filename, id);
            }
        }
    }

    pub async fn process_single_pdf(
        &self,
        pdf_data: &Bytes,
        filename: &str,
        id: Uuid,
    ) -> Option<(String, Value)> {
        let pdf_text = match pdf_extract::extract_text_from_mem(pdf_data) {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("Failed to extract text from PDF for filename {}, id {}: {}", filename, id, e);
                return None;
            }
        };
        
        tracing::debug!("PDF Text Contents for filename {}, id {}: {}", filename, id, pdf_text);
        
        let response = generate_structure_from_pdf(
            &pdf_text, 
            &self.state.http_client, 
            &self.state.openai_api_key, 
            &self.state.resume_schema
        ).await;

        match response {
            Ok(r) => {
                if let Some(choice) = r.choices.first() {
                    let raw_content = &choice.message.content;
                    match from_str::<Value>(raw_content) {
                        Ok(parsed_json) => {
                             let pretty_json = to_string(&parsed_json).unwrap_or_else(|_| "{}".to_string());
                             tracing::info!("LLM-generated JSON received for filename {}, id {}", filename, id);
                             tracing::debug!("LLM-generated JSON for filename {}, id {}: {}", filename, id, pretty_json);
                             Some((pdf_text, parsed_json))
                        },
                        Err(e) => {
                            tracing::error!("LLM returned invalid JSON for filename {}, id {}: {}", filename, id, e);
                            tracing::debug!("Raw content: {}", raw_content);
                            None
                        }
                    }
                }
                else {
                    tracing::error!("No choices returned from LLM for filename {}, id {}", filename, id);
                    None
                }
            }
            Err(e) => {
                tracing::error!("LLM request failed for filename {}, id {}: {:#?}", filename, id, e);
                None
            }
        }
    }

    pub async fn update_resume_record(&self, id: Uuid, text: String, structured_json : Value) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!("UPDATE resumes SET text = $1, structured = $2 WHERE id = $3", text, structured_json, id)
            .execute(&self.state.pool)
            .await
    }

    pub async fn handle_batch_extraction(&self, id: Uuid, filename: String) {
        let zip_data = match self.state.supabase.storage()
            .download("zip-archives", &filename)
            .await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to download zip for filename {}, id {}: {}", filename, id, e);
                    return;
                }
            };
            
        let mut tmp_file = match tempfile() {
             Ok(f) => f,
             Err(e) => {
                 tracing::error!("Failed to create tempfile: {}", e);
                 return;
             }
        };
        
        if let Err(e) = tmp_file.write_all(&zip_data) {
             tracing::error!("Failed to write to tempfile: {}", e);
             return;
        }
        
        let mut archive = match zip::ZipArchive::new(tmp_file) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("Failed to create zip archive: {}", e);
                return;
            }
        };
        
        tracing::info!("Successfully extracted zip archive with {} files", archive.len());
        for i in 0..archive.len() {
            let mut file = match archive.by_index(i) {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("Failed to read file at index {} in zip: {}", i, e);
                    continue;
                }
            };
            
            if file.is_dir() || !file.name().ends_with(".pdf") {
                continue;
            }

            let mut pdf_buffer = Vec::new();
            if let Err(e) = file.read_to_end(&mut pdf_buffer) {
                tracing::error!("Failed to read file {} to buffer: {}", file.name(), e);
                continue;
            }
            
            let pdf_bytes = Bytes::from(pdf_buffer);
            let pdf_name = file.name().to_string();
            let upload_path = format!("{}_{}", filename, pdf_name);
            
            let supabase = self.state.supabase.clone();
            let semaphore = self.state.semaphore.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Semaphore closed");
                let options = supabase::storage::FileOptions {
                    cache_control: None,
                    content_type: Some("application/pdf".to_string()),
                    upsert: false,
                };

                match supabase.storage()
                    .upload("resumes", &upload_path, pdf_bytes, Some(options))
                    .await {
                        Ok(_) => tracing::info!("Successfully re-uploaded extracted PDF: {}", upload_path),
                        Err(e) => tracing::error!("Failed to upload extracted PDF {}: {}", upload_path, e),
                    }
            });
        }
    }
}
