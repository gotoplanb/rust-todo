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
use tracing::{info, instrument, warn, Span};
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

#[derive(Debug, Deserialize)]
struct BatchCreateRequest {
    todos: Vec<CreateTodoRequest>,
}

#[derive(Debug, Serialize)]
struct BatchCreateResponse {
    created: Vec<Todo>,
    total: usize,
}

type TodoStore = Arc<Mutex<HashMap<Uuid, Todo>>>;

#[derive(Clone)]
struct AppState {
    todos: TodoStore,
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    todos_count: usize,
}

// Simulated database operations with tracing
#[instrument(skip(store))]
async fn simulate_db_create(store: &TodoStore, todo: Todo) -> Todo {
    info!("Creating todo in database");
    
    // Simulate database latency
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    
    let mut todos = store.lock().unwrap();
    todos.insert(todo.id, todo.clone());
    
    info!("Todo created successfully");
    todo
}

#[instrument(skip(store))]
async fn simulate_db_list(store: &TodoStore) -> Vec<Todo> {
    info!("Listing todos from database");
    
    // Simulate database latency
    tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
    
    let todos = store.lock().unwrap();
    let todo_list: Vec<Todo> = todos.values().cloned().collect();
    
    info!(count = todo_list.len(), "Retrieved todos from database");
    todo_list
}

#[instrument(skip(store))]
async fn simulate_db_get(store: &TodoStore, id: Uuid) -> Option<Todo> {
    info!("Getting todo from database");
    
    // Simulate database latency
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    let todos = store.lock().unwrap();
    let result = todos.get(&id).cloned();
    
    match &result {
        Some(_) => info!("Todo found"),
        None => warn!("Todo not found"),
    }
    
    result
}

#[instrument(skip(store))]
async fn simulate_db_update(store: &TodoStore, todo: Todo) -> Option<Todo> {
    info!("Updating todo in database");
    
    // Simulate database latency
    tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
    
    let mut todos = store.lock().unwrap();
    if todos.contains_key(&todo.id) {
        todos.insert(todo.id, todo.clone());
        info!("Todo updated successfully");
        Some(todo)
    } else {
        warn!("Todo not found for update");
        None
    }
}

#[instrument(skip(store))]
async fn simulate_db_delete(store: &TodoStore, id: Uuid) -> bool {
    info!("Deleting todo from database");
    
    // Simulate database latency
    tokio::time::sleep(tokio::time::Duration::from_millis(18)).await;
    
    let mut todos = store.lock().unwrap();
    let existed = todos.remove(&id).is_some();
    
    if existed {
        info!("Todo deleted successfully");
    } else {
        warn!("Todo not found for deletion");
    }
    
    existed
}

// Simulated external service calls
#[instrument]
async fn send_notification(todo_id: Uuid, event_type: &str) {
    info!(todo_id = %todo_id, event = event_type, "Sending notification");
    
    // Simulate webhook call
    let webhook_span = tracing::info_span!("webhook_call");
    let _enter = webhook_span.enter();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
    info!("Webhook notification sent");
    
    drop(_enter);
    
    // Simulate email service
    let email_span = tracing::info_span!("email_service");
    let _enter = email_span.enter();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(60)).await;
    info!("Email notification sent");
}

#[instrument]
async fn send_batch_summary(count: usize) {
    info!(batch_count = count, "Sending batch summary");
    
    let aggregation_span = tracing::info_span!("aggregation_service");
    let _enter = aggregation_span.enter();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(45)).await;
    info!("Batch summary sent");
}

// API Handlers
#[instrument(skip(state))]
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    info!("Health check requested");
    
    let todo_list = simulate_db_list(&state.todos).await;
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "0.2.0".to_string(),
        todos_count: todo_list.len(),
    })
}

#[instrument(skip(state))]
async fn list_todos(State(state): State<AppState>) -> Json<Vec<Todo>> {
    info!("Listing todos");
    
    let todos = simulate_db_list(&state.todos).await;
    Json(todos)
}

#[instrument(skip(state), fields(title = %payload.title, todo_id))]
async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodoRequest>,
) -> Json<Todo> {
    info!("Creating todo");
    
    let todo = Todo {
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        completed: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Record todo ID in current span
    Span::current().record("todo_id", &tracing::field::display(&todo.id));
    
    // Create in database
    let created_todo = simulate_db_create(&state.todos, todo).await;
    
    // Send notification (don't block the response)
    let notification_span = tracing::info_span!("send_notifications");
    let _guard = notification_span.enter();
    
    tokio::spawn(send_notification(created_todo.id, "created"));
    
    info!("Todo created successfully");
    Json(created_todo)
}

#[instrument(skip(state), fields(batch_size = payload.todos.len()))]
async fn create_batch(
    State(state): State<AppState>,
    Json(payload): Json<BatchCreateRequest>,
) -> Json<BatchCreateResponse> {
    info!(count = payload.todos.len(), "Creating batch of todos");
    
    let mut created_todos = Vec::new();
    let total = payload.todos.len();
    
    // Process each todo in the batch
    for (index, todo_req) in payload.todos.into_iter().enumerate() {
        let batch_item_span = tracing::info_span!("batch_item", item_index = index);
        let _guard = batch_item_span.enter();
        
        info!("Processing batch item {}", index);
        
        let todo = Todo {
            id: Uuid::new_v4(),
            title: todo_req.title,
            description: todo_req.description,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let created = simulate_db_create(&state.todos, todo).await;
        created_todos.push(created);
    }
    
    // Send batch summary
    tokio::spawn(send_batch_summary(created_todos.len()));
    
    info!(created_count = created_todos.len(), "Batch creation completed");
    
    Json(BatchCreateResponse {
        total,
        created: created_todos,
    })
}

#[instrument(skip(state), fields(todo_id = %id))]
async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Getting todo");
    
    match simulate_db_get(&state.todos, id).await {
        Some(todo) => {
            info!("Todo retrieved");
            Ok(Json(todo))
        }
        None => {
            warn!("Todo not found");
            Err((StatusCode::NOT_FOUND, "Todo not found"))
        }
    }
}

#[instrument(skip(state, payload), fields(todo_id = %id))]
async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodoRequest>,
) -> impl IntoResponse {
    info!("Updating todo");
    
    // First, get the existing todo
    let mut todo = match simulate_db_get(&state.todos, id).await {
        Some(t) => t,
        None => {
            warn!("Todo not found for update");
            return Err((StatusCode::NOT_FOUND, "Todo not found"));
        }
    };
    
    let was_completed = todo.completed;
    
    // Update fields
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
    
    // Update in database
    let updated_todo = match simulate_db_update(&state.todos, todo).await {
        Some(t) => t,
        None => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update todo"));
        }
    };
    
    // Send completion notification if todo was just completed
    if !was_completed && updated_todo.completed {
        tokio::spawn(send_notification(updated_todo.id, "completed"));
    }
    
    info!("Todo updated successfully");
    Ok(Json(updated_todo))
}

#[instrument(skip(state), fields(todo_id = %id))]
async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Deleting todo");
    
    let deleted = simulate_db_delete(&state.todos, id).await;
    
    if deleted {
        info!("Todo deleted");
        Ok(StatusCode::NO_CONTENT)
    } else {
        warn!("Todo not found for deletion");
        Err((StatusCode::NOT_FOUND, "Todo not found"))
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
                    opentelemetry::KeyValue::new("service.version", "0.2.0"),
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
        .route("/todos/batch", axum::routing::post(create_batch))
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