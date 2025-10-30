use mongodb::bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRoute {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub method: String,
    pub path: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub body: Option<JsonValue>,
    pub created_at: BsonDateTime,
}

impl RequestLog {
    pub fn now(method: String, path: String, body: Option<JsonValue>) -> Self {
        let now = OffsetDateTime::now_utc();
        let millis: i64 = now.unix_timestamp() * 1000;
        Self {
            id: None,
            method,
            path,
            body,
            created_at: BsonDateTime::from_millis(millis),
        }
    }
}


