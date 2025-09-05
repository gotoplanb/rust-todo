use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BatchCreateRequest {
    pub todos: Vec<CreateTodoRequest>,
}

#[derive(Debug, Serialize)]
pub struct BatchCreateResponse {
    pub created: Vec<Todo>,
    pub total: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct BatchDeleteResponse {
    pub deleted: usize,
    pub not_found: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct DeleteCompletedResponse {
    pub deleted_count: usize,
}