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
```

## Run
```bash
cargo run
```
Or watch
```bash
cargo watch -w src -x run
```

## Create mock (JSON/text)
```bash
curl -X POST http://localhost:3000/mocks \
  -H "Content-Type: application/json" \
  -d '{"method":"GET","path":"/ping","response_type":"text","response_data":"pong"}'
```

## Create mock (file)
```bash
curl -X POST http://localhost:3000/mocks \
  -F "method=GET" \
  -F "path=/download/manual" \
  -F "response_type=file" \
  -F "file=@./manual.pdf"
```

## Test
```bash
curl http://localhost:3000/ping
```