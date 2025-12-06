use std::env;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::{Value};

// pub struct Resume {
//     education: Vec<EducationEntry>,
//     skills: Vec<String>,
//     experience: Vec<ExperienceEntry>,
// }
//
// pub struct EducationEntry {
//     school: String,
//     degree_type: String,
//     degree_title: String,
//     grad_date: String,
//     gpa: f32
// }
//
// pub struct ExperienceEntry {
//     role: String,
//     years_of_experience: f32
// }

#[derive(Serialize)]
pub struct LLMRequest {
    model: String,
    messages: [Message; 2],
    response_format: ResponseFormat
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")] // This tells serde to use "type": "json_schema"
pub enum ResponseFormat {
    #[serde(rename = "json_schema")]
    JsonSchema {
        json_schema: JsonSchemaDefinition,
    },
    #[serde(rename = "text")]
    Text,
}

#[derive(Serialize, Debug)]
pub struct JsonSchemaDefinition {
    pub name: String,
    pub strict: bool,
    pub schema: Value
}
#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
}

const OPENAI_MODEL: &str = "gpt-5-nano";

pub async fn generate_structure_from_pdf(resume_text : &str) -> Result<ChatCompletionResponse, reqwest::Error> {
    dotenv().ok();
    let system_prompt = "You are a resume conversion assistant. Extract information from the user's resume text and format it into the given structure.".to_string();
    let user_prompt = resume_text.to_string();
    let raw_schema_string = include_str!("../resume_schema.json");
    let parsed_schema : Value = serde_json::from_str(raw_schema_string).expect("Invalid JSON Schema File");
    let request = LLMRequest {
        model: OPENAI_MODEL.to_string(),
        messages: [
            Message {
                role: "system".to_string(),
                content: system_prompt
        },
            Message {
                role: "user".to_string(),
                content: user_prompt
        }],
        response_format: ResponseFormat::JsonSchema {
            json_schema: JsonSchemaDefinition {
                name: "resume_data_structuring".to_string(),
                strict: false,
                schema: parsed_schema,
            },
        },
    };

    reqwest::Client::new().post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(env::var("OPENAI_API_KEY").expect("No OPENAI_API_KEY environment variable"))
        .json(&request)
        .send()
        .await?
        .json::<ChatCompletionResponse>()
        .await
}