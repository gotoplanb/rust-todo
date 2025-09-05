use async_trait::async_trait;
use tracing::{info, instrument, warn, Span};
use uuid::Uuid;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Notification service error: {0}")]
    NotificationFailed(String),
    
    #[error("External API timeout")]
    Timeout,
    
    #[error("Rate limited")]
    RateLimited,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_created_notification(&self, todo_id: Uuid, title: &str) -> Result<(), ServiceError>;
    async fn send_completed_notification(&self, todo_id: Uuid, title: &str) -> Result<(), ServiceError>;
    async fn send_batch_summary(&self, count: usize) -> Result<(), ServiceError>;
}

pub struct MockNotificationService;

impl MockNotificationService {
    pub fn new() -> Self {
        Self
    }
    
    #[instrument(skip(self), fields(service = "external_api", latency_ms))]
    async fn simulate_api_call(&self, endpoint: &str) -> Result<(), ServiceError> {
        // Simulate API latency  
        let delay_ms = {
            let mut rng = rand::thread_rng();
            rng.gen_range(50..250)
        };
        let delay = Duration::from_millis(delay_ms);
        Span::current().record("latency_ms", delay.as_millis());
        
        info!(endpoint, "Calling external API");
        tokio::time::sleep(delay).await;
        
        // Generate random values before async operations
        let fail_chance = {
            let mut rng = rand::thread_rng();
            rng.gen::<f32>()
        };
        
        // Simulate occasional failures (10% failure rate)
        if fail_chance < 0.1 {
            warn!(endpoint, "External API call failed");
            return Err(ServiceError::NotificationFailed("Random failure".to_string()));
        }
        
        // Simulate occasional rate limiting (5% rate)
        if fail_chance < 0.15 {
            warn!(endpoint, "Rate limited by external API");
            return Err(ServiceError::RateLimited);
        }
        
        info!(endpoint, latency_ms = delay.as_millis(), "External API call successful");
        Ok(())
    }
}

#[async_trait]
impl NotificationService for MockNotificationService {
    #[instrument(skip(self), fields(notification.type = "todo_created", todo.id = %todo_id, todo.title = %title))]
    async fn send_created_notification(&self, todo_id: Uuid, title: &str) -> Result<(), ServiceError> {
        info!("Sending todo created notification");
        
        // Simulate webhook call
        {
            let _webhook_span = tracing::info_span!("webhook_call", url = "https://api.slack.com/webhook").entered();
            self.simulate_api_call("/webhook/todo-created").await?;
        }
        
        // Simulate email service call
        {
            let _email_span = tracing::info_span!("email_service", recipient = "team@example.com").entered();
            self.simulate_api_call("/email/send").await?;
        }
        
        info!("Notifications sent successfully");
        Ok(())
    }
    
    #[instrument(skip(self), fields(notification.type = "todo_completed", todo.id = %todo_id, todo.title = %title))]
    async fn send_completed_notification(&self, todo_id: Uuid, title: &str) -> Result<(), ServiceError> {
        info!("Sending todo completed notification");
        
        // Simulate analytics event
        {
            let _analytics_span = tracing::info_span!("analytics_event", event = "todo.completed").entered();
            self.simulate_api_call("/analytics/track").await?;
        }
        
        info!("Completion notification sent");
        Ok(())
    }
    
    #[instrument(skip(self), fields(notification.type = "batch_summary", batch.count = count))]
    async fn send_batch_summary(&self, count: usize) -> Result<(), ServiceError> {
        info!(count, "Sending batch summary notification");
        
        // Simulate aggregation service call
        {
            let _aggregation_span = tracing::info_span!("aggregation_service").entered();
            self.simulate_api_call("/aggregate/batch-summary").await?;
        }
        
        info!("Batch summary sent");
        Ok(())
    }
}

use rand::Rng;