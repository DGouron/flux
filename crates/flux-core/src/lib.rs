//! Flux core library
//!
//! Contains domain types and port definitions (traits) for the Flux application.
//! This crate has no knowledge of infrastructure concerns.

pub mod config;
pub mod domain;
pub mod i18n;
pub mod ports;
pub mod secrets;
pub mod state;

pub use config::{
    Config, ConfigError, DigestConfig, DistractionConfig, FocusConfig, GeneralConfig,
    NotificationConfig, NotificationUrgency, Profile, TrayConfig,
};
pub use domain::{
    AppUsage, DigestStats, DistractionSuggestion, FocusMode, Provider, ReviewAction, ReviewEvent,
    Session, SessionId, SessionMetrics, SuggestionReason, SuggestionReport, WeekStats,
};
pub use i18n::{Language, Translator, UnsupportedLanguageError};
pub use ports::{
    AppTrackingRepository, AppTrackingRepositoryError, ReviewActivityGateway, ReviewGatewayError,
    SessionMetricsRepository, SessionMetricsRepositoryError, SessionRepository,
    SessionRepositoryError,
};
pub use secrets::{
    resolve_github_credentials, resolve_gitlab_credentials, ProviderCredentials, SecretsError,
};
pub use state::AppState;
