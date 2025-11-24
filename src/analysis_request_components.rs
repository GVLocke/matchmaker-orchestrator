mod date_format;
mod supabase;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use supabase::authenticate_supabase_client;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    title: String,
    team: Option<String>,
    manager: Option<String>,
    priority: u8,
    #[serde(with = "date_format")]
    start_date: DateTime<Utc>,
    #[serde(with = "date_format")]
    end_date: DateTime<Utc>,
    requirements: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Intern {
    name: String,
    email: String,
    document_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    projects: Vec<Project>,
    interns: Vec<Intern>,
}

pub async fn get_analysis() {
    match authenticate_supabase_client().await {
        Ok(client) => {
            match client.storage().download("resumes/", "James_Keys_CV.pdf").await {
                Ok(pdf) => {
                    println!("Downloaded resume. Size: {}", pdf.len());
                }
                Err(e) => {
                    println!("Failed to download resume. Error: {}", e);
                }
            }
        }
        Err(error) => {
            println!("Failed to authenticate using the supabase client. Error: {}", error);
        }
    }
}

// pub async fn vehicle_post(Json(mut v) : Json<Vehicle>) -> Json<Vehicle> {
//     println!("Manufacturer: {}, model: {}, year: {}", v.manufacturer, v.model, v.year);
//     v.id = Some(uuid::Uuid::new_v4().to_string());
//     Json::from(v)
// }
