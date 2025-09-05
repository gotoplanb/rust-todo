mod models;
mod repository;
mod external_service;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use models::*;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use repository::{SqliteTodoRepository, TodoRepository};
use external_service::{MockNotificationService, NotificationService};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::{error, info, instrument, warn, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    repository: Arc<dyn TodoRepository>,
    notification_service: Arc<dyn NotificationService>,
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    database: String,
}

#[instrument(skip(state))]
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    info!("Health check requested");
    
    // Check database connectivity
    let db_status = match state.repository.list().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "0.2.0".to_string(),
        database: db_status.to_string(),
    })
}

#[instrument(skip(state))]
async fn list_todos(State(state): State<AppState>) -> impl IntoResponse {
    info!("Listing todos");
    
    match state.repository.list().await {
        Ok(todos) => {
            info!(count = todos.len(), "Retrieved todos");
            Ok(Json(todos))
        }
        Err(e) => {
            error!(error = %e, "Failed to list todos");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve todos"))
        }
    }
}

#[instrument(skip(state), fields(title = %payload.title))]
async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodoRequest>,
) -> impl IntoResponse {
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
    Span::current().record("todo.id", &tracing::field::display(&todo.id));
    
    // Create in database
    let created_todo = match state.repository.create(todo).await {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "Failed to create todo");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create todo"));
        }
    };
    
    // Send notification (don't fail the request if this fails)
    let notification_span = tracing::info_span!("send_notifications");
    let _guard = notification_span.enter();
    
    if let Err(e) = state.notification_service
        .send_created_notification(created_todo.id, &created_todo.title)
        .await 
    {
        warn!(error = %e, "Failed to send notification, continuing anyway");
    }
    
    info!("Todo created successfully");
    Ok(Json(created_todo))
}

#[instrument(skip(state), fields(batch_size = payload.todos.len()))]
async fn create_batch(
    State(state): State<AppState>,
    Json(payload): Json<BatchCreateRequest>,
) -> impl IntoResponse {
    info!(count = payload.todos.len(), "Creating batch of todos");
    
    let todos: Vec<Todo> = payload
        .todos
        .into_iter()
        .map(|req| Todo {
            id: Uuid::new_v4(),
            title: req.title,
            description: req.description,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .collect();
    
    let total = todos.len();
    
    // Create todos in batch
    match state.repository.create_batch(todos).await {
        Ok(created) => {
            info!(created_count = created.len(), "Batch creation successful");
            
            // Send batch summary notification
            let _ = state.notification_service.send_batch_summary(created.len()).await;
            
            Ok(Json(BatchCreateResponse {
                total,
                created,
                errors: vec![],
            }))
        }
        Err(e) => {
            error!(error = %e, "Batch creation failed");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Batch creation failed"))
        }
    }
}

#[instrument(skip(state), fields(todo.id = %id))]
async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Getting todo");
    
    match state.repository.get(id).await {
        Ok(todo) => {
            info!("Todo retrieved");
            Ok(Json(todo))
        }
        Err(repository::RepositoryError::NotFound(_)) => {
            warn!("Todo not found");
            Err((StatusCode::NOT_FOUND, "Todo not found"))
        }
        Err(e) => {
            error!(error = %e, "Failed to get todo");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve todo"))
        }
    }
}

#[instrument(skip(state, payload), fields(todo.id = %id))]
async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodoRequest>,
) -> impl IntoResponse {
    info!("Updating todo");
    
    // First, get the existing todo
    let mut todo = match state.repository.get(id).await {
        Ok(t) => t,
        Err(repository::RepositoryError::NotFound(_)) => {
            warn!("Todo not found for update");
            return Err((StatusCode::NOT_FOUND, "Todo not found"));
        }
        Err(e) => {
            error!(error = %e, "Failed to get todo for update");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update todo"));
        }
    };
    
    // Track if we're completing a todo
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
    let updated_todo = match state.repository.update(todo).await {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "Failed to update todo");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update todo"));
        }
    };
    
    // Send completion notification if todo was just completed
    if !was_completed && updated_todo.completed {
        let _ = state.notification_service
            .send_completed_notification(updated_todo.id, &updated_todo.title)
            .await;
    }
    
    info!("Todo updated successfully");
    Ok(Json(updated_todo))
}

#[instrument(skip(state), fields(todo.id = %id))]
async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Deleting todo");
    
    match state.repository.delete(id).await {
        Ok(()) => {
            info!("Todo deleted");
            Ok(StatusCode::NO_CONTENT)
        }
        Err(repository::RepositoryError::NotFound(_)) => {
            warn!("Todo not found for deletion");
            Err((StatusCode::NOT_FOUND, "Todo not found"))
        }
        Err(e) => {
            error!(error = %e, "Failed to delete todo");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete todo"))
        }
    }
}

#[instrument(skip(state))]
async fn delete_completed(State(state): State<AppState>) -> impl IntoResponse {
    info!("Deleting all completed todos");
    
    match state.repository.delete_completed().await {
        Ok(count) => {
            info!(deleted_count = count, "Completed todos deleted");
            Ok(Json(DeleteCompletedResponse {
                deleted_count: count,
            }))
        }
        Err(e) => {
            error!(error = %e, "Failed to delete completed todos");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete completed todos"))
        }
    }
}

// Validation middleware
#[instrument(skip_all)]
async fn validate_request(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    info!(method = %method, uri = %uri, "Validating request");
    
    // Add artificial validation delay for demo
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    
    let response = next.run(req).await;
    
    info!("Request validated and processed");
    response
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

    // Initialize repository
    let repository = SqliteTodoRepository::new("sqlite:todos.db")
        .await
        .expect("Failed to connect to database");
    
    // Initialize services
    let notification_service = MockNotificationService::new();
    
    let state = AppState {
        repository: Arc::new(repository),
        notification_service: Arc::new(notification_service),
    };
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/batch", post(create_batch))
        .route("/todos/completed", delete(delete_completed))
        .route("/todos/:id", get(get_todo).put(update_todo).delete(delete_todo))
        .layer(middleware::from_fn(validate_request))
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