use async_trait::async_trait;
use chrono::{DateTime, Utc};
use flux_core::{ReviewActivityGateway, ReviewEvent, ReviewGatewayError};

use super::dto::{GitLabEvent, GitLabMergeRequest};

pub struct GitLabReviewGateway {
    base_url: String,
    token: String,
    user_id: u64,
}

impl GitLabReviewGateway {
    pub fn new(base_url: String, token: String, user_id: u64) -> Self {
        Self {
            base_url,
            token,
            user_id,
        }
    }

    fn fetch_events(&self, since: DateTime<Utc>) -> Result<Vec<GitLabEvent>, ReviewGatewayError> {
        let url = format!("{}/api/v4/users/{}/events", self.base_url, self.user_id);

        let response = ureq::get(&url)
            .set("PRIVATE-TOKEN", &self.token)
            .query("target_type", "merge_request")
            .query("after", &since.format("%Y-%m-%d").to_string())
            .query("per_page", "100")
            .call()
            .map_err(|error| self.handle_error(error))?;

        response
            .into_json::<Vec<GitLabEvent>>()
            .map_err(|error| ReviewGatewayError::Parse {
                message: error.to_string(),
            })
    }

    fn fetch_pending_merge_requests(&self) -> Result<Vec<GitLabMergeRequest>, ReviewGatewayError> {
        let url = format!("{}/api/v4/merge_requests", self.base_url);

        let response = ureq::get(&url)
            .set("PRIVATE-TOKEN", &self.token)
            .query("scope", "all")
            .query("state", "opened")
            .query("reviewer_id", &self.user_id.to_string())
            .query("per_page", "100")
            .call()
            .map_err(|error| self.handle_error(error))?;

        response
            .into_json::<Vec<GitLabMergeRequest>>()
            .map_err(|error| ReviewGatewayError::Parse {
                message: error.to_string(),
            })
    }

    fn handle_error(&self, error: ureq::Error) -> ReviewGatewayError {
        match error {
            ureq::Error::Status(401, _) | ureq::Error::Status(403, _) => {
                ReviewGatewayError::Authentication
            }
            ureq::Error::Status(429, _) => ReviewGatewayError::RateLimited { retry_after: 60 },
            ureq::Error::Status(code, response) => ReviewGatewayError::Network {
                message: format!("HTTP {}: {}", code, response.status_text()),
            },
            ureq::Error::Transport(transport) => ReviewGatewayError::Network {
                message: transport.to_string(),
            },
        }
    }
}

#[async_trait]
impl ReviewActivityGateway for GitLabReviewGateway {
    async fn get_activity_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<ReviewEvent>, ReviewGatewayError> {
        let base_url = self.base_url.clone();
        let token = self.token.clone();
        let user_id = self.user_id;

        let gateway = GitLabReviewGateway::new(base_url.clone(), token, user_id);

        let events = tokio::task::spawn_blocking(move || gateway.fetch_events(since))
            .await
            .map_err(|error| ReviewGatewayError::Network {
                message: format!("task join error: {}", error),
            })??;

        tracing::debug!(
            event_count = events.len(),
            "fetched GitLab events since {}",
            since
        );

        Ok(events
            .into_iter()
            .filter(|event| event.created_at >= since)
            .filter(|event| event.target_type.as_deref() == Some("MergeRequest"))
            .map(|event| event.into_review_event(&base_url))
            .collect())
    }

    async fn get_pending_reviews(&self) -> Result<Vec<ReviewEvent>, ReviewGatewayError> {
        let base_url = self.base_url.clone();
        let token = self.token.clone();
        let user_id = self.user_id;

        let gateway = GitLabReviewGateway::new(base_url, token, user_id);

        let merge_requests = tokio::task::spawn_blocking(move || gateway.fetch_pending_merge_requests())
            .await
            .map_err(|error| ReviewGatewayError::Network {
                message: format!("task join error: {}", error),
            })??;

        tracing::debug!(
            merge_request_count = merge_requests.len(),
            "fetched pending GitLab merge requests"
        );

        Ok(merge_requests
            .into_iter()
            .map(|merge_request| merge_request.into_review_event())
            .collect())
    }

    fn provider_name(&self) -> &'static str {
        "GitLab"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_gateway_with_correct_configuration() {
        let gateway = GitLabReviewGateway::new(
            "https://gitlab.com".to_string(),
            "test-token".to_string(),
            12345,
        );

        assert_eq!(gateway.base_url, "https://gitlab.com");
        assert_eq!(gateway.user_id, 12345);
        assert_eq!(gateway.provider_name(), "GitLab");
    }
}
