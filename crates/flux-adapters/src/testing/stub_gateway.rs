use async_trait::async_trait;
use chrono::{DateTime, Utc};
use flux_core::{ReviewActivityGateway, ReviewEvent, ReviewGatewayError};
use std::sync::Mutex;

pub struct StubReviewGateway {
    events: Mutex<Vec<ReviewEvent>>,
}

impl StubReviewGateway {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn with_events(events: Vec<ReviewEvent>) -> Self {
        Self {
            events: Mutex::new(events),
        }
    }

    pub fn given_events(&self, events: Vec<ReviewEvent>) {
        let mut guard = self.events.lock().unwrap();
        *guard = events;
    }
}

impl Default for StubReviewGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReviewActivityGateway for StubReviewGateway {
    async fn get_activity_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<ReviewEvent>, ReviewGatewayError> {
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
            .filter(|event| event.timestamp >= since)
            .cloned()
            .collect())
    }

    async fn get_pending_reviews(&self) -> Result<Vec<ReviewEvent>, ReviewGatewayError> {
        let events = self.events.lock().unwrap();
        Ok(events.clone())
    }

    fn provider_name(&self) -> &'static str {
        "Stub"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use flux_core::{Provider, ReviewAction};

    fn create_test_event(id: &str, hours_ago: i64) -> ReviewEvent {
        ReviewEvent {
            identifier: id.to_string(),
            author: "test-author".to_string(),
            repository: "test-repo".to_string(),
            title: "Test MR".to_string(),
            action: ReviewAction::Opened,
            timestamp: Utc::now() - Duration::hours(hours_ago),
            url: "https://example.com/mr/1".to_string(),
            provider: Provider::GitLab,
        }
    }

    #[tokio::test]
    async fn returns_empty_when_no_events() {
        let gateway = StubReviewGateway::new();
        let since = Utc::now() - Duration::hours(24);

        let result = gateway.get_activity_since(since).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn filters_events_by_timestamp() {
        let old_event = create_test_event("old", 48);
        let recent_event = create_test_event("recent", 1);

        let gateway = StubReviewGateway::with_events(vec![old_event, recent_event]);
        let since = Utc::now() - Duration::hours(24);

        let result = gateway.get_activity_since(since).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].identifier, "recent");
    }

    #[tokio::test]
    async fn returns_all_events_for_pending_reviews() {
        let events = vec![create_test_event("1", 1), create_test_event("2", 2)];
        let gateway = StubReviewGateway::with_events(events);

        let result = gateway.get_pending_reviews().await.unwrap();

        assert_eq!(result.len(), 2);
    }
}
