//! Flux core library
//!
//! Contains domain types and port definitions (traits) for the Flux application.
//! This crate has no knowledge of infrastructure concerns.

pub mod domain;
pub mod ports;

pub use domain::{Provider, ReviewAction, ReviewEvent};
pub use ports::{ReviewActivityGateway, ReviewGatewayError};
