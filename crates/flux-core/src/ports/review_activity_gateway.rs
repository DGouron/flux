use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::ReviewEvent;

#[derive(Error, Debug, Clone)]
pub enum ReviewGatewayError {
    #[error("erreur réseau: {message}")]
    Network { message: String },

    #[error("authentification échouée")]
    Authentication,

    #[error("rate limit atteint, réessayer dans {retry_after} secondes")]
    RateLimited { retry_after: u64 },

    #[error("erreur de parsing: {message}")]
    Parse { message: String },

    #[error("provider non configuré: {provider}")]
    NotConfigured { provider: String },
}

#[async_trait]
pub trait ReviewActivityGateway: Send + Sync {
    async fn get_activity_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<ReviewEvent>, ReviewGatewayError>;

    async fn get_pending_reviews(&self) -> Result<Vec<ReviewEvent>, ReviewGatewayError>;

    fn provider_name(&self) -> &'static str;
}
