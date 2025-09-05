use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::{error, info, instrument, warn, Span};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::time::Duration;
use crate::Todo;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Todo not found: {0}")]
    NotFound(Uuid),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn create(&self, todo: Todo) -> Result<Todo, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Todo, RepositoryError>;
    async fn list(&self) -> Result<Vec<Todo>, RepositoryError>;
    async fn update(&self, todo: Todo) -> Result<Todo, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn create_batch(&self, todos: Vec<Todo>) -> Result<Vec<Todo>, RepositoryError>;
    async fn delete_completed(&self) -> Result<usize, RepositoryError>;
}

pub struct SqliteTodoRepository {
    pool: Pool<Sqlite>,
}

impl SqliteTodoRepository {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Run migrations
        sqlx::query(include_str!("../migrations/001_create_todos.sql"))
            .execute(&pool)
            .await?;
            
        Ok(Self { pool })
    }
    
    #[instrument(skip(self), fields(operation = "simulate_latency"))]
    async fn simulate_db_latency(&self) {
        // Simulate realistic database latency for demo purposes
        let delay_ms = {
            let mut rng = rand::thread_rng();
            rng.gen_range(10..60)
        };
        let delay = Duration::from_millis(delay_ms);
        tokio::time::sleep(delay).await;
        info!(delay_ms = delay.as_millis(), "Simulated DB latency");
    }
}

#[async_trait]
impl TodoRepository for SqliteTodoRepository {
    #[instrument(skip(self, todo), fields(todo.id = %todo.id, todo.title = %todo.title, db.operation = "INSERT"))]
    async fn create(&self, todo: Todo) -> Result<Todo, RepositoryError> {
        info!("Creating todo in database");
        self.simulate_db_latency().await;
        
        let created_at = todo.created_at.to_rfc3339();
        let updated_at = todo.updated_at.to_rfc3339();
        
        let id_str = todo.id.to_string();
        let result = sqlx::query(
            r#"
            INSERT INTO todos (id, title, description, completed, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#
        )
        .bind(&id_str)
        .bind(&todo.title)
        .bind(&todo.description)
        .bind(todo.completed)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await;
        
        match result {
            Ok(_) => {
                info!("Todo created successfully in database");
                Ok(todo)
            }
            Err(e) => {
                error!(error = %e, "Failed to create todo in database");
                Err(RepositoryError::Database(e))
            }
        }
    }
    
    #[instrument(skip(self), fields(todo.id = %id, db.operation = "SELECT"))]
    async fn get(&self, id: Uuid) -> Result<Todo, RepositoryError> {
        info!("Fetching todo from database");
        self.simulate_db_latency().await;
        
        let id_str = id.to_string();
        let row = sqlx::query_as::<_, (String, String, Option<String>, bool, String, String)>(
            r#"
            SELECT id, title, description, completed, created_at, updated_at
            FROM todos
            WHERE id = ?1
            "#
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some((id_str, title, description, completed, created_at, updated_at)) => {
                info!("Todo found in database");
                Ok(Todo {
                    id: Uuid::parse_str(&id_str).unwrap(),
                    title,
                    description,
                    completed,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            }
            None => {
                warn!("Todo not found in database");
                Err(RepositoryError::NotFound(id))
            }
        }
    }
    
    #[instrument(skip(self), fields(db.operation = "SELECT_ALL"))]
    async fn list(&self) -> Result<Vec<Todo>, RepositoryError> {
        info!("Listing all todos from database");
        self.simulate_db_latency().await;
        
        let rows = sqlx::query_as::<_, (String, String, Option<String>, bool, String, String)>(
            r#"
            SELECT id, title, description, completed, created_at, updated_at
            FROM todos
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        let todos: Vec<Todo> = rows
            .into_iter()
            .map(|(id_str, title, description, completed, created_at, updated_at)| Todo {
                id: Uuid::parse_str(&id_str).unwrap(),
                title,
                description,
                completed,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect();
        
        info!(count = todos.len(), "Fetched todos from database");
        Ok(todos)
    }
    
    #[instrument(skip(self, todo), fields(todo.id = %todo.id, db.operation = "UPDATE"))]
    async fn update(&self, todo: Todo) -> Result<Todo, RepositoryError> {
        info!("Updating todo in database");
        self.simulate_db_latency().await;
        
        let updated_at = todo.updated_at.to_rfc3339();
        
        let id_str = todo.id.to_string();
        let result = sqlx::query(
            r#"
            UPDATE todos
            SET title = ?2, description = ?3, completed = ?4, updated_at = ?5
            WHERE id = ?1
            "#
        )
        .bind(&id_str)
        .bind(&todo.title)
        .bind(&todo.description)
        .bind(todo.completed)
        .bind(&updated_at)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            warn!("Todo not found for update");
            Err(RepositoryError::NotFound(todo.id))
        } else {
            info!("Todo updated successfully");
            Ok(todo)
        }
    }
    
    #[instrument(skip(self), fields(todo.id = %id, db.operation = "DELETE"))]
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        info!("Deleting todo from database");
        self.simulate_db_latency().await;
        
        let id_str = id.to_string();
        let result = sqlx::query(
            r#"
            DELETE FROM todos
            WHERE id = ?1
            "#
        )
        .bind(&id_str)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            warn!("Todo not found for deletion");
            Err(RepositoryError::NotFound(id))
        } else {
            info!("Todo deleted successfully");
            Ok(())
        }
    }
    
    #[instrument(skip(self, todos), fields(batch_size = todos.len(), db.operation = "BATCH_INSERT"))]
    async fn create_batch(&self, todos: Vec<Todo>) -> Result<Vec<Todo>, RepositoryError> {
        info!(count = todos.len(), "Creating batch of todos");
        
        let current_span = Span::current();
        let mut created_todos = Vec::new();
        
        for (index, todo) in todos.into_iter().enumerate() {
            let span = tracing::info_span!(
                parent: &current_span,
                "batch_item",
                item_index = index,
                todo.id = %todo.id
            );
            let _guard = span.enter();
            
            info!("Processing batch item");
            let created = self.create(todo).await?;
            created_todos.push(created);
        }
        
        info!(created_count = created_todos.len(), "Batch creation completed");
        Ok(created_todos)
    }
    
    #[instrument(skip(self), fields(db.operation = "DELETE_COMPLETED"))]
    async fn delete_completed(&self) -> Result<usize, RepositoryError> {
        info!("Deleting all completed todos");
        self.simulate_db_latency().await;
        
        let result = sqlx::query(
            r#"
            DELETE FROM todos
            WHERE completed = true
            "#
        )
        .execute(&self.pool)
        .await?;
        
        let deleted_count = result.rows_affected() as usize;
        info!(deleted_count, "Deleted completed todos");
        Ok(deleted_count)
    }
}

// Add this for random delays
use rand::Rng;