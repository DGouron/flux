//! Flux core library
//!
//! Contains domain types and port definitions (traits) for the Flux application.
//! This crate has no knowledge of infrastructure concerns.

pub mod config;
pub mod domain;
pub mod i18n;
pub mod ports;
pub mod secrets;

pub use config::{
    Config, ConfigError, DistractionConfig, FocusConfig, GeneralConfig, NotificationConfig,
    NotificationUrgency, TrayConfig,
};
pub use domain::{AppUsage, FocusMode, Provider, ReviewAction, ReviewEvent, Session, SessionId};
pub use i18n::{Language, Translator, UnsupportedLanguageError};
pub use ports::{
    AppTrackingRepository, AppTrackingRepositoryError, ReviewActivityGateway, ReviewGatewayError,
    SessionRepository, SessionRepositoryError,
};
pub use secrets::{
    resolve_github_credentials, resolve_gitlab_credentials, ProviderCredentials, SecretsError,
};
