use async_trait::async_trait;
use chrono::{DateTime, Utc};
use flux_core::{ReviewActivityGateway, ReviewEvent, ReviewGatewayError};

pub struct FailingReviewGateway {
    error: ReviewGatewayError,
}

impl FailingReviewGateway {
    pub fn network_error() -> Self {
        Self {
            error: ReviewGatewayError::Network {
                message: "connection refused".to_string(),
            },
        }
    }

    pub fn authentication_error() -> Self {
        Self {
            error: ReviewGatewayError::Authentication,
        }
    }

    pub fn rate_limited(retry_after: u64) -> Self {
        Self {
            error: ReviewGatewayError::RateLimited { retry_after },
        }
    }

    pub fn with_error(error: ReviewGatewayError) -> Self {
        Self { error }
    }
}

#[async_trait]
impl ReviewActivityGateway for FailingReviewGateway {
    async fn get_activity_since(
        &self,
        _since: DateTime<Utc>,
    ) -> Result<Vec<ReviewEvent>, ReviewGatewayError> {
        Err(self.error.clone())
    }

    async fn get_pending_reviews(&self) -> Result<Vec<ReviewEvent>, ReviewGatewayError> {
        Err(self.error.clone())
    }

    fn provider_name(&self) -> &'static str {
        "Failing"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_network_error() {
        let gateway = FailingReviewGateway::network_error();

        let result = gateway.get_pending_reviews().await;

        assert!(matches!(result, Err(ReviewGatewayError::Network { .. })));
    }

    #[tokio::test]
    async fn returns_authentication_error() {
        let gateway = FailingReviewGateway::authentication_error();

        let result = gateway.get_pending_reviews().await;

        assert!(matches!(result, Err(ReviewGatewayError::Authentication)));
    }

    #[tokio::test]
    async fn returns_rate_limited_error() {
        let gateway = FailingReviewGateway::rate_limited(120);

        let result = gateway.get_pending_reviews().await;

        match result {
            Err(ReviewGatewayError::RateLimited { retry_after }) => {
                assert_eq!(retry_after, 120);
            }
            _ => panic!("expected RateLimited error"),
        }
    }
}
