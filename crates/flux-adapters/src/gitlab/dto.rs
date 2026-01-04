use chrono::{DateTime, Utc};
use flux_core::{Provider, ReviewAction, ReviewEvent};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GitLabEvent {
    pub id: u64,
    pub action_name: String,
    pub target_type: Option<String>,
    pub target_id: Option<u64>,
    pub target_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub project_id: Option<u64>,
}

impl GitLabEvent {
    pub fn into_review_event(self, base_url: &str) -> ReviewEvent {
        let project_id = self.project_id.unwrap_or(0);
        let target_id = self.target_id.unwrap_or(0);

        ReviewEvent {
            identifier: format!("gitlab-event-{}", self.id),
            author: String::new(),
            repository: format!("project-{}", project_id),
            title: self.target_title.unwrap_or_default(),
            action: Self::parse_action(&self.action_name),
            timestamp: self.created_at,
            url: format!(
                "{}/projects/{}/merge_requests/{}",
                base_url, project_id, target_id
            ),
            provider: Provider::GitLab,
        }
    }

    fn parse_action(action_name: &str) -> ReviewAction {
        match action_name {
            "commented on" => ReviewAction::Commented,
            "approved" => ReviewAction::Approved,
            "opened" => ReviewAction::Opened,
            "merged" => ReviewAction::Merged,
            "closed" => ReviewAction::Closed,
            _ => ReviewAction::Commented,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GitLabMergeRequest {
    pub iid: u64,
    pub title: String,
    pub state: String,
    pub project_id: u64,
    pub web_url: String,
    pub updated_at: DateTime<Utc>,
    pub author: GitLabUser,
}

#[derive(Deserialize, Debug)]
pub struct GitLabUser {
    pub username: String,
}

impl GitLabMergeRequest {
    pub fn into_review_event(self) -> ReviewEvent {
        ReviewEvent {
            identifier: format!("gitlab-mr-{}-{}", self.project_id, self.iid),
            author: self.author.username,
            repository: format!("project-{}", self.project_id),
            title: self.title,
            action: Self::parse_state(&self.state),
            timestamp: self.updated_at,
            url: self.web_url,
            provider: Provider::GitLab,
        }
    }

    fn parse_state(state: &str) -> ReviewAction {
        match state {
            "opened" => ReviewAction::Opened,
            "merged" => ReviewAction::Merged,
            "closed" => ReviewAction::Closed,
            _ => ReviewAction::Opened,
        }
    }
}
