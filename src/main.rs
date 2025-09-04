use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateTodoRequest {
    title: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateTodoRequest {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

type TodoStore = Arc<Mutex<HashMap<Uuid, Todo>>>;

#[derive(Clone)]
struct AppState {
    todos: TodoStore,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[instrument]
async fn health_check() -> Json<HealthResponse> {
    info!("Health check requested");
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
    })
}

#[instrument(skip(state))]
async fn list_todos(State(state): State<AppState>) -> Json<Vec<Todo>> {
    let todos = state.todos.lock().unwrap();
    let todo_count = todos.len();
    let todo_list: Vec<Todo> = todos.values().cloned().collect();
    info!(count = todo_count, "Listing todos");
    Json(todo_list)
}

#[instrument(skip(state), fields(title = %payload.title, todo_id))]
async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodoRequest>,
) -> Json<Todo> {
    let now = Utc::now();
    let todo = Todo {
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        completed: false,
        created_at: now,
        updated_at: now,
    };
    
    tracing::Span::current().record("todo_id", &tracing::field::display(&todo.id));
    
    let mut todos = state.todos.lock().unwrap();
    todos.insert(todo.id, todo.clone());
    
    info!(todo.id = %todo.id, todo.title = %todo.title, "Todo created");
    Json(todo)
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let todos = state.todos.lock().unwrap();
    
    match todos.get(&id) {
        Some(todo) => Ok(Json(todo.clone())),
        None => Err((StatusCode::NOT_FOUND, "Todo not found")),
    }
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodoRequest>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();
    
    match todos.get_mut(&id) {
        Some(todo) => {
            if let Some(title) = payload.title {
                todo.title = title;
            }
            if let Some(description) = payload.description {
                todo.description = Some(description);
            }
            if let Some(completed) = payload.completed {
                todo.completed = completed;
            }
            todo.updated_at = Utc::now();
            
            Ok(Json(todo.clone()))
        }
        None => Err((StatusCode::NOT_FOUND, "Todo not found")),
    }
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();
    
    match todos.remove(&id) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err((StatusCode::NOT_FOUND, "Todo not found")),
    }
}

async fn init_tracing() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "todo-api"),
                    opentelemetry::KeyValue::new("service.version", "0.1.0"),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::TokioCurrentThread)
        .expect("Failed to install OpenTelemetry tracer");

    let telemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer.tracer("todo-api"));

    tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .compact(),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing().await;

    let state = AppState {
        todos: Arc::new(Mutex::new(HashMap::new())),
    };
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/:id", get(get_todo).put(update_todo).delete(delete_todo))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🚀 Server starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
