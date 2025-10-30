use std::sync::Arc;

use mongodb::{options::ClientOptions, Client, Collection, Database};

use crate::models::{MockRoute, RequestLog};

#[derive(Clone)]
pub struct AppState {
    pub _db: Database,
    pub mocks: Collection<MockRoute>,
    pub requests: Collection<RequestLog>,
}

impl AppState {
    pub async fn connect_from_env() -> anyhow::Result<Arc<Self>> {
        let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let db_name = std::env::var("MONGODB_DB").unwrap_or_else(|_| "mock-services".to_string());
        let client_options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(&db_name);
        let mocks = db.collection::<MockRoute>("mocks");
        let requests = db.collection::<RequestLog>("requests");
        Ok(Arc::new(Self { _db: db, mocks, requests }))
    }
}
