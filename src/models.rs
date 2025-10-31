use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRoute {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub method: String,
    pub path: String,
    #[serde(default = "default_http_status_code")]
    pub http_status_code: u16,
    pub response_type: ResponseType,
    // For json: raw JSON. For text: raw string. For file: file path on disk.
    pub response_data: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] 
pub enum ResponseType {
    Json,
    Text,
    File,
}

pub fn default_http_status_code() -> u16 { 200 }


