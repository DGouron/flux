use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewEvent {
    pub identifier: String,
    pub author: String,
    pub repository: String,
    pub title: String,
    pub action: ReviewAction,
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub provider: Provider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewAction {
    Opened,
    Commented,
    Approved,
    ChangesRequested,
    Merged,
    Closed,
}

impl ReviewAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewAction::Opened => "opened",
            ReviewAction::Commented => "commented",
            ReviewAction::Approved => "approved",
            ReviewAction::ChangesRequested => "changes_requested",
            ReviewAction::Merged => "merged",
            ReviewAction::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    GitLab,
    GitHub,
    Bitbucket,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::GitLab => "gitlab",
            Provider::GitHub => "github",
            Provider::Bitbucket => "bitbucket",
        }
    }
}
