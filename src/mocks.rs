use std::path::PathBuf;

use axum::{body::Body, http::{header, StatusCode}, response::Response, Json};
use mongodb::bson::doc;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use tracing::{error, info};

use crate::{db::AppState, models::{MockRoute, ResponseType}};

pub async fn upsert_mock_json(state: &AppState, mut mock: MockRoute) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Normalize method & path
    mock.method = mock.method.to_uppercase();
    if !mock.path.starts_with('/') { mock.path = format!("/{}", mock.path); }

    let filter = doc!{"method": &mock.method, "path": &mock.path};
    let opts = mongodb::options::ReplaceOptions::builder().upsert(true).build();
    match state.mocks.replace_one(filter, &mock, opts).await {
        Ok(res) => {
            info!(matched = res.matched_count, modified = res.modified_count, upserted = ?res.upserted_id, "mock upserted");
            Ok(Json(json!({"status":"ok"})))
        }
        Err(e) => {
            error!(error = ?e, "replace_one failed");
            Err(internal_error())
        }
    }
}

pub async fn upsert_mock_file(state: &AppState, method: String, path: String, filename: String, bytes: Vec<u8>, status_code: u16) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let method = method.to_uppercase();
    let path = if path.starts_with('/') { path } else { format!("/{}", path) };

    let safe_name = format!("{}_{}", Uuid::new_v4(), filename);
    let dest: PathBuf = ["src", "uploads", &safe_name].iter().collect();
    if let Some(parent) = dest.parent() { let _ = tokio::fs::create_dir_all(parent).await; }
    let mut f = tokio::fs::File::create(&dest).await.map_err(|_| internal_error())?;
    f.write_all(&bytes).await.map_err(|_| internal_error())?;

    let mock = MockRoute {
        id: None,
        method,
        path,
        http_status_code: status_code,
        response_type: ResponseType::File,
        response_data: json!(dest.to_string_lossy().to_string()),
    };
    let filter = doc!{"method": &mock.method, "path": &mock.path};
    let opts = mongodb::options::ReplaceOptions::builder().upsert(true).build();
    match state.mocks.replace_one(filter, &mock, opts).await {
        Ok(res) => {
            info!(matched = res.matched_count, modified = res.modified_count, upserted = ?res.upserted_id, "file mock upserted");
            Ok(Json(json!({"status":"ok","file": mock.response_data})))
        }
        Err(e) => {
            error!(error = ?e, "replace_one failed");
            Err(internal_error())
        }
    }
}

pub async fn serve_mock(state: &AppState, method: &str, path: &str) -> Response {
    let key_method = method.to_uppercase();
    let key_path = if path.starts_with('/') { path.to_string() } else { format!("/{}", path) };
    let filter = doc!{"method": &key_method, "path": &key_path};
    match state.mocks.find_one(filter, None).await {
        Ok(Some(mock)) => match mock.response_type {
            ResponseType::Json => {
                let status = StatusCode::from_u16(mock.http_status_code).unwrap_or(StatusCode::OK);
                (status, axum::Json(mock.response_data)).into_response()
            }
            ResponseType::Text => {
                let text = mock.response_data.as_str().unwrap_or("").to_string();
                let status = StatusCode::from_u16(mock.http_status_code).unwrap_or(StatusCode::OK);
                Response::builder().status(status).header(header::CONTENT_TYPE, "text/plain").body(Body::from(text)).unwrap()
            }
            ResponseType::File => {
                let path = mock.response_data.as_str().unwrap_or("");
                let file = tokio::fs::read(path).await;
                match file {
                    Ok(bytes) => {
                        let status = StatusCode::from_u16(mock.http_status_code).unwrap_or(StatusCode::OK);
                        Response::builder()
                        .status(status)
                        .header(header::CONTENT_TYPE, "application/octet-stream")
                        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or("download")))
                        .body(Body::from(bytes))
                        .unwrap()
                    },
                    Err(_) => Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("file not found")).unwrap(),
                }
            }
        },
        _ => {
            let fallback = json!({"path": key_path, "method": key_method, "status": "mocked"});
            axum::Json(fallback).into_response()
        }
    }
}

fn internal_error() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"internal"})))
}

use axum::response::IntoResponse;


