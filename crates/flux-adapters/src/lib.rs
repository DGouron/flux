//! Flux adapters - Infrastructure implementations
//!
//! This crate contains concrete implementations of the ports defined in flux-core.
//! It bridges the domain logic with external services like GitLab, GitHub, etc.

pub mod gitlab;
pub mod sqlite;
pub mod testing;

pub use gitlab::GitLabReviewGateway;
pub use sqlite::SqliteSessionRepository;
pub use testing::{FailingReviewGateway, StubReviewGateway};
