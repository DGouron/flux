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
    Config, ConfigError, FocusConfig, GeneralConfig, NotificationConfig, NotificationUrgency,
    TrayConfig,
};
pub use domain::{FocusMode, Provider, ReviewAction, ReviewEvent, Session, SessionId};
pub use i18n::{Language, Translator, UnsupportedLanguageError};
pub use ports::{
    ReviewActivityGateway, ReviewGatewayError, SessionRepository, SessionRepositoryError,
};
pub use secrets::{
    resolve_github_credentials, resolve_gitlab_credentials, ProviderCredentials, SecretsError,
};
