# Todo API with Advanced OpenTelemetry Tracing

A Rust REST API built with Axum featuring **multi-level distributed tracing** using OpenTelemetry. Perfect for learning both Rust and modern observability patterns.

## 🚀 Features

### Core Functionality
- ✅ Full CRUD operations for todos
- ✅ Batch operations for bulk processing
- ✅ SQLite database with repository pattern
- ✅ Simulated external service calls
- ✅ Validation middleware

### Advanced Tracing
- 🔍 **Multi-level span hierarchies** - See exactly how requests flow
- 📊 **Database operation tracing** - Every query creates a child span
- 🌐 **External service simulation** - Webhook and email notification spans
- ⏱️ **Latency simulation** - Realistic timing for learning
- 🎯 **Contextual attributes** - IDs, counts, and operation types

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
# Run comprehensive trace tests
./test_traces.sh

# Or test individual endpoints
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/todos
```

### 4. View Traces in Jaeger
Open **http://localhost:16686** and explore:
- Service: `todo-api`
- Look for complex trace hierarchies
- Click on spans to see attributes and timing

## API Endpoints

### Basic CRUD
- `GET /health` - Health check with DB connectivity
- `GET /todos` - List all todos
- `POST /todos` - Create todo
- `GET /todos/{id}` - Get specific todo
- `PUT /todos/{id}` - Update todo
- `DELETE /todos/{id}` - Delete todo

### Advanced Operations
- `POST /todos/batch` - Create multiple todos (generates nested spans)
- `DELETE /todos/completed` - Delete all completed todos

## 📈 Trace Hierarchy Example

```
HTTP POST /todos/batch
├── validate_request (middleware)
├── create_batch (handler)
│   ├── batch_item_0
│   │   ├── database.INSERT
│   │   └── simulate_latency
│   ├── batch_item_1
│   │   ├── database.INSERT
│   │   └── simulate_latency
│   └── send_batch_summary
│       └── aggregation_service
│           └── external_api

HTTP POST /todos
├── validate_request
├── database.INSERT
│   └── simulate_latency
└── send_notifications
    ├── webhook_call
    │   └── external_api
    └── email_service
        └── external_api
```

## 🏗️ Architecture

```
src/
├── main.rs              # HTTP handlers and server setup
├── models.rs            # Data structures
├── repository.rs        # Database layer with tracing
└── external_service.rs  # Simulated external calls
```

### Key Components

1. **Repository Pattern** - Database operations with automatic span creation
2. **Service Layer** - External API simulation with realistic latencies
3. **Middleware** - Request validation with tracing
4. **Dependency Injection** - Using `Arc<dyn Trait>` for flexibility

## 🎓 Learning Concepts

### Rust Patterns
- **Ownership & Borrowing** - `Arc<Mutex<>>` for shared state
- **Async/Await** - Tokio runtime with tracing context
- **Error Handling** - Custom error types with `thiserror`
- **Trait Objects** - `dyn TodoRepository` for abstraction
- **Pattern Matching** - Elegant error handling

### OpenTelemetry Concepts
- **Spans** - Units of work with timing
- **Traces** - Complete request flows
- **Context Propagation** - Maintaining trace IDs across async boundaries
- **Attributes** - Structured metadata on spans
- **Instrumentation** - Both automatic and manual

## 🔧 Configuration

### Environment Variables
- `RUST_LOG=info` - Enable info-level logging
- `RUST_LOG=debug` - See detailed trace information

### Jaeger Configuration
The `docker-compose.yml` sets up:
- Jaeger UI: http://localhost:16686
- OTLP gRPC: localhost:4317
- OTLP HTTP: localhost:4318

## 📚 Advanced Usage

### Creating Complex Traces
```bash
# Create batch of todos (multiple nested spans)
curl -X POST http://127.0.0.1:3000/todos/batch \
  -H "Content-Type: application/json" \
  -d '{
    "todos": [
      {"title": "Task 1", "description": "Description 1"},
      {"title": "Task 2", "description": "Description 2"},
      {"title": "Task 3", "description": "Description 3"}
    ]
  }'
```

### Triggering Notification Spans
```bash
# Complete a todo (triggers analytics event)
curl -X PUT http://127.0.0.1:3000/todos/{id} \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'
```

## 🚦 Observability Benefits

1. **Performance Analysis** - Identify slow database queries or API calls
2. **Error Tracking** - See exactly where failures occur
3. **System Understanding** - Visualize request flow through components
4. **Debugging** - Contextual information for troubleshooting
5. **Learning Tool** - Perfect for understanding distributed systems

## 🛠️ Development

### Building
```bash
cargo build
```

### Testing
```bash
./test_crud.sh    # Basic CRUD operations
./test_traces.sh  # Complex tracing scenarios
```

### Adding New Traces
1. Add `#[instrument]` to functions
2. Use `tracing::info_span!()` for manual spans
3. Record attributes with `Span::current().record()`
4. Create child spans for nested operations

## 📖 Resources

- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Axum Framework](https://github.com/tokio-rs/axum)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

---

Built as a comprehensive learning project for Rust and OpenTelemetry! 

Perfect for understanding:
- How distributed tracing works
- Modern Rust patterns
- Observability in microservices
- Async programming with context propagation

🦀 + 🔍 = ❤️