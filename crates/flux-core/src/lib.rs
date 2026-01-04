//! Flux core library
//!
//! Contains domain types and port definitions (traits) for the Flux application.
//! This crate has no knowledge of infrastructure concerns.

pub mod config;
pub mod domain;
pub mod ports;
pub mod secrets;

pub use config::{Config, ConfigError, FocusConfig, NotificationConfig, NotificationUrgency};
pub use domain::{FocusMode, Provider, ReviewAction, ReviewEvent, Session, SessionId};
pub use ports::{ReviewActivityGateway, ReviewGatewayError, SessionRepository, SessionRepositoryError};
pub use secrets::{resolve_gitlab_credentials, resolve_github_credentials, ProviderCredentials, SecretsError};
