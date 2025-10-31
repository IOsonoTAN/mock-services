use std::sync::Arc;

use mongodb::{options::ClientOptions, Client, Collection, Database};
use std::env;

// AWS S3
#[derive(Clone)]
pub struct S3Config {
    pub bucket: String,
    pub bucket_url: Option<String>,
    pub cloudfront_domain: Option<String>,
}

#[derive(Clone)]
pub struct S3State {
    pub client: aws_sdk_s3::Client,
    pub config: S3Config,
}

use crate::models::MockRoute;

#[derive(Clone)]
pub struct AppState {
    pub _db: Database,
    pub mocks: Collection<MockRoute>,
    pub s3: Option<S3State>,
}

impl AppState {
    pub async fn connect_from_env() -> anyhow::Result<Arc<Self>> {
        let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let db_name = env::var("MONGODB_DB").unwrap_or_else(|_| "mock-services".to_string());
        let client_options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(&db_name);
        let mocks = db.collection::<MockRoute>("mocks");

        // Initialize S3 if bucket is configured
        let s3 = if let Ok(bucket) = env::var("AWS_S3_BUCKET") {
            let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            let client = aws_sdk_s3::Client::new(&aws_config);
            let bucket_url = env::var("AWS_S3_BUCKET_URL").ok();
            let cloudfront_domain = env::var("AWS_S3_CLOUDFRONT_DOMAIN").ok();
            Some(S3State { client, config: S3Config { bucket, bucket_url, cloudfront_domain } })
        } else {
            None
        };

        Ok(Arc::new(Self { _db: db, mocks, s3 }))
    }
}
