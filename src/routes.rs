use std::sync::Arc;

use axum::{
    extract::{FromRequest, Multipart, Path, Request, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{any, post, get},
    Json, Router,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use tracing::info;

use crate::{db::AppState, models::{MockRoute, RequestLog, ResponseType}, mocks};

#[derive(Clone)]
struct SharedState(Arc<AppState>);

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/mocks", post(create_mock))
        .route("/*path", any(catch_all))
        .with_state(SharedState(state))
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status":"ok"}))).into_response()
}

#[derive(Deserialize, Serialize)]
struct MockJsonRequest {
    method: String,
    path: String,
    response_type: ResponseType,
    response_data: JsonValue,
}

async fn create_mock(State(state): State<SharedState>, req: Request) -> impl IntoResponse {
    let content_type = req.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
    if content_type.starts_with("application/json") {
        let body = req.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|_| StatusCode::BAD_REQUEST).unwrap_or_default();
        let parsed: Result<MockJsonRequest, _> = serde_json::from_slice(&bytes);
        match parsed {
            Ok(req) => {
                let mock = MockRoute {
                    id: None,
                    method: req.method,
                    path: req.path,
                    response_type: req.response_type,
                    response_data: req.response_data,
                };
                match mocks::upsert_mock_json(&state.0, mock).await {
                    Ok(json) => (StatusCode::OK, json).into_response(),
                    Err(err) => err.into_response(),
                }
            }
            Err(_) => (StatusCode::BAD_REQUEST, Json(json!({"error":"invalid json"}))).into_response(),
        }
    } else if content_type.starts_with("multipart/form-data") {
        let mut multipart = match Multipart::from_request(req, &()).await {
            Ok(m) => m,
            Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error":"invalid multipart"}))).into_response(),
        };
        let mut method: Option<String> = None;
        let mut path: Option<String> = None;
        let mut file_name: Option<String> = None;
        let mut file_bytes: Vec<u8> = Vec::new();
        while let Some(field) = multipart.next_field().await.unwrap_or(None) {
            let name = field.name().map(|s| s.to_string());
            match name.as_deref() {
                Some("method") => method = Some(field.text().await.unwrap_or_default()),
                Some("path") => path = Some(field.text().await.unwrap_or_default()),
                Some("file") => {
                    file_name = field.file_name().map(|s| s.to_string()).or(Some("upload.bin".to_string()));
                    let bytes = field.bytes().await.unwrap_or_default();
                    file_bytes = bytes.to_vec();
                }
                Some("response_type") => { /* ignored for multipart, forced to file */ }
                _ => {}
            }
        }
        if let (Some(m), Some(p), Some(fname)) = (method, path, file_name) {
            match mocks::upsert_mock_file(&state.0, m, p, fname, file_bytes).await {
                Ok(json) => (StatusCode::OK, json).into_response(),
                Err(err) => err.into_response(),
            }
        } else {
            (StatusCode::BAD_REQUEST, Json(json!({"error":"missing fields"}))).into_response()
        }
    } else {
        (StatusCode::UNSUPPORTED_MEDIA_TYPE, Json(json!({"error":"unsupported content-type"}))).into_response()
    }
}

async fn catch_all(Path(path): Path<String>, State(state): State<SharedState>, method: axum::http::Method, req: Request) -> impl IntoResponse {
    // Log request
    let body_json: Option<JsonValue> = match axum::body::to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(bytes) if !bytes.is_empty() => serde_json::from_slice(&bytes).ok(),
        _ => None,
    };
    let log = RequestLog::now(method.to_string(), format!("/{}", path), body_json);
    let _ = state.0.requests.insert_one(log, None).await;

    info!(method = %method, path = %path, "incoming request");
    mocks::serve_mock(&state.0, &method.to_string(), &format!("/{}", path)).await
}


