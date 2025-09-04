# Todo API with OpenTelemetry Tracing

A Rust REST API built with Axum and OpenTelemetry for distributed tracing.

## Features

- ✅ Full CRUD operations for todos
- ✅ OpenTelemetry tracing integration
- ✅ Jaeger visualization support
- ✅ Structured logging
- ✅ Thread-safe in-memory storage

## Quick Start

### 1. Start Jaeger (for trace visualization)
```bash
docker-compose up -d
```

### 2. Run the API server
```bash
RUST_LOG=info cargo run
```

### 3. Test the API
```bash
# Run all CRUD tests
./test_crud.sh

# Or test individual endpoints
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/todos
curl -X POST http://127.0.0.1:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn Rust", "description": "Build awesome APIs"}'
```

### 4. View Traces
Open **http://localhost:16686** in your browser to see traces in Jaeger UI:
- Service: `todo-api`
- Operation: Look for HTTP requests and function spans
- Traces show: Request flow, timing, structured data

## API Endpoints

- `GET /health` - Health check
- `GET /todos` - List all todos
- `POST /todos` - Create todo
- `GET /todos/{id}` - Get specific todo
- `PUT /todos/{id}` - Update todo
- `DELETE /todos/{id}` - Delete todo

## Tracing Features

The API includes comprehensive tracing:
- **HTTP requests** automatically traced
- **Function-level spans** with `#[instrument]`
- **Structured fields** (todo IDs, titles, counts)
- **Error handling** with proper status codes
- **OTLP export** to Jaeger

## Learning Rust Concepts

This project demonstrates:
- **Ownership & Borrowing** with `Arc<Mutex<>>`
- **Error Handling** with `Result<T,E>`
- **Async Programming** with Tokio
- **Type Safety** with `Option<T>`
- **Trait System** with Serde
- **Pattern Matching** with `match`

---

Built as a learning project for Rust and OpenTelemetry! 🦀