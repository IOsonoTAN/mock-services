# Dynamic Mock Service (Rust + Axum + MongoDB)

## Start up the dependency
```bash
docker compose up -d
```

## Create .env variables
```bash
PORT=3000
MONGODB_URI=mongodb://localhost:27017
MONGODB_DB=mock-services
# Optional: enable S3 uploads (if unset, files are stored locally)
AWS_S3_BUCKET=your-bucket-name
AWS_REGION=ap-southeast-1
# Standard AWS credentials resolution is used (env vars, shared config/credentials, IAM role)
# e.g.
# AWS_ACCESS_KEY_ID=...
# AWS_SECRET_ACCESS_KEY=...
# AWS_SESSION_TOKEN=...
# Optional public URLs (when set, file responses redirect instead of streaming):
# If CloudFront is set, it is preferred.
AWS_S3_BUCKET_URL=https://your-bucket.s3.amazonaws.com
AWS_S3_CLOUDFRONT_DOMAIN=d111111abcdef8.cloudfront.net
```

## Run
```bash
cargo start
```
Or watch (requires `cargo install cargo-watch`)
```bash
cargo dev
```

## Create mock (JSON/text)
```bash
curl -X POST http://localhost:3000/mocks \
  -H "Content-Type: application/json" \
  -d '{"method":"GET","path":"/ping","http_status_code":200,"response_type":"text","response_data":"pong"}'
```

## Create mock (file)
```bash
curl -X POST http://localhost:3000/mocks \
  -F "method=GET" \
  -F "path=/download/manual" \
  -F "http_status_code=400" \
  -F "response_type=file" \
  -F "file=@./manual.pdf"
```

When `AWS_S3_BUCKET` is set, the uploaded file is stored in S3 and the DB stores `{ bucket, key }`.

Serving behavior priority for files:
- If `AWS_S3_CLOUDFRONT_DOMAIN` is set: 307 redirect to `https://<domain>/<key>`
- Else if `AWS_S3_BUCKET_URL` is set: 307 redirect to `<bucket_url>/<key>`
- Else: stream object from S3
- If S3 is not configured: store and stream from `src/uploads/`

## Test
```bash
curl -i http://localhost:3000/
curl -i http://localhost:3000/ping
curl -i http://localhost:3000/download/manual
```

## Update an existing mock (PATCH)
```bash
curl -X PATCH http://localhost:3000/mocks \
  -H "Content-Type: application/json" \
  -d '{"method":"GET","path":"/ping","http_status_code":201}'
```

## Docker
```bash
docker build -t mock-services .
docker run --rm -p 3000:3000 \
  -e MONGODB_URI="mongodb://host.docker.internal:27017" \
  mock-services
```