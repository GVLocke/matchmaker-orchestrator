use std::io::{Read, Write};
use axum::body::Bytes;
use serde_json::{Value, from_str, to_string};
use tempfile::tempfile;
use uuid::Uuid;
use aws_sdk_s3::primitives::ByteStream;
use crate::AppState;
use crate::requests::openai::generate_structure_from_pdf;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

pub struct ResumeService {
    state: AppState,
}

impl ResumeService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn process_and_update_resume(&self, id: Uuid, filename: String) {
        let _permit = self.state.semaphore.acquire().await.expect("Semaphore closed");
        
        // Mark as processing
        let _ = self.update_resume_status(id, JobStatus::Processing, None).await;

        // Download
        let pdf_data = match self.state.s3_client.get_object()
            .bucket("resumes")
            .key(&filename)
            .send()
            .await {
                Ok(output) => {
                    match output.body.collect().await {
                        Ok(data) => data.into_bytes(),
                        Err(e) => {
                            let err_msg = format!("Failed to collect pdf body: {}", e);
                            tracing::error!("{}, filename {}, id {}", err_msg, filename, id);
                            let _ = self.update_resume_status(id, JobStatus::Failed, Some(err_msg)).await;
                            return;
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to download pdf: {:#?}", e);
                    tracing::error!("{}, filename {}, id {}", err_msg, filename, id);
                    let _ = self.update_resume_status(id, JobStatus::Failed, Some(err_msg)).await;
                    return;
                }
            };

        // Parse and process
        match self.process_single_pdf(&pdf_data, &filename, id).await {
            Some((pdf_text, parsed_json)) => {
                match self.update_resume_record(id, pdf_text, parsed_json).await {
                    Ok(_) => {
                        tracing::info!("Resume record {} (filename: {}) updated successfully", id, filename);
                        let _ = self.update_resume_status(id, JobStatus::Completed, None).await;
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to update database record: {}", e);
                        tracing::error!("{}, filename {}, id {}", err_msg, filename, id);
                        let _ = self.update_resume_status(id, JobStatus::Failed, Some(err_msg)).await;
                    }
                }
            }
            None => {
                 let err_msg = "PDF processing or LLM parsing failed".to_string();
                 tracing::warn!("{}, filename {}, id {}", err_msg, filename, id);
                 let _ = self.update_resume_status(id, JobStatus::Failed, Some(err_msg)).await;
            }
        }
    }

    async fn update_resume_status(&self, id: Uuid, status: JobStatus, error_message: Option<String>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE resumes SET status = $1, error_message = $2 WHERE id = $3",
            status as JobStatus,
            error_message,
            id
        )
        .execute(&self.state.pool)
        .await?;
        Ok(())
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

    async fn update_zip_status(&self, id: Uuid, status: JobStatus, error_message: Option<String>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE zip_archives SET status = $1, error_message = $2 WHERE id = $3",
            status as JobStatus,
            error_message,
            id
        )
        .execute(&self.state.pool)
        .await?;
        Ok(())
    }

    pub async fn handle_batch_extraction(&self, id: Uuid, filename: String) {
        let _permit = self.state.semaphore.acquire().await.expect("Semaphore closed");

        // Mark as processing
        let _ = self.update_zip_status(id, JobStatus::Processing, None).await;

        let zip_data = match self.state.s3_client.get_object()
            .bucket("zip-archives")
            .key(&filename)
            .send()
            .await {
                Ok(output) => {
                    match output.body.collect().await {
                        Ok(data) => data.into_bytes(),
                        Err(e) => {
                            let err_msg = format!("Failed to collect zip body: {}", e);
                            tracing::error!("{}, filename {}, id {}", err_msg, filename, id);
                            let _ = self.update_zip_status(id, JobStatus::Failed, Some(err_msg)).await;
                            return;
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to download zip: {:#?}", e);
                    tracing::error!("{}, filename {}, id {}", err_msg, filename, id);
                    let _ = self.update_zip_status(id, JobStatus::Failed, Some(err_msg)).await;
                    return;
                }
            };
            
        let mut tmp_file = match tempfile() {
             Ok(f) => f,
             Err(e) => {
                 let err_msg = format!("Failed to create tempfile: {}", e);
                 tracing::error!("{}", err_msg);
                 let _ = self.update_zip_status(id, JobStatus::Failed, Some(err_msg)).await;
                 return;
             }
        };
        
        if let Err(e) = tmp_file.write_all(&zip_data) {
             let err_msg = format!("Failed to write to tempfile: {}", e);
             tracing::error!("{}", err_msg);
             let _ = self.update_zip_status(id, JobStatus::Failed, Some(err_msg)).await;
             return;
        }
        
        let mut archive = match zip::ZipArchive::new(tmp_file) {
            Ok(a) => a,
            Err(e) => {
                let err_msg = format!("Failed to create zip archive: {}", e);
                tracing::error!("{}", err_msg);
                let _ = self.update_zip_status(id, JobStatus::Failed, Some(err_msg)).await;
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
            
            let s3_client = self.state.s3_client.clone();
            let semaphore = self.state.semaphore.clone();
            let pool = self.state.pool.clone(); // Clone pool for DB update

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Semaphore closed");

                match s3_client.put_object()
                    .bucket("resumes")
                    .key(&upload_path)
                    .body(ByteStream::from(pdf_bytes))
                    .content_type("application/pdf")
                    // .metadata("zip_id", zip_id_str) // Metadata propagation is unreliable, handled via DB update below
                    .send()
                    .await {
                        Ok(_) => {
                             tracing::info!("Successfully re-uploaded extracted PDF: {}", upload_path);
                             // Explicitly link the resume to the zip archive
                             // We use a retry loop briefly in case the trigger hasn't fired yet (race condition)
                             let mut retries = 0;
                             while retries < 3 {
                                 match sqlx::query!("UPDATE resumes SET zip_id = $1 WHERE filename = $2", id, upload_path)
                                     .execute(&pool)
                                     .await 
                                 {
                                     Ok(result) => {
                                         if result.rows_affected() > 0 {
                                             tracing::info!("Linked resume {} to zip_id {}", upload_path, id);
                                             break;
                                         } else {
                                             tracing::warn!("Resume row not found for linking (retry {}): {}", retries, upload_path);
                                         }
                                     },
                                     Err(e) => {
                                         tracing::error!("Failed to link resume {} to zip_id {}: {}", upload_path, id, e);
                                     }
                                 }
                                 retries += 1;
                                 tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                             }
                        },
                        Err(e) => tracing::error!("Failed to upload extracted PDF {}: {}", upload_path, e),
                    }
            });
        }
        
        let _ = self.update_zip_status(id, JobStatus::Completed, None).await;
    }
}
