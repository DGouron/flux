//! Flux protocol definitions for CLI-daemon communication
//!
//! This crate defines the IPC protocol between the flux CLI and daemon.
//! All types are serializable with bincode for efficient binary communication.

use serde::{Deserialize, Serialize};

pub use flux_core::FocusMode;

/// Requests sent from CLI to daemon
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    /// Start a new focus session
    StartSession {
        /// Duration in minutes (None = use default from config)
        duration: Option<u64>,
        /// Focus mode (None = use default)
        mode: Option<FocusMode>,
    },
    /// Stop the current focus session
    StopSession,
    /// Pause the current focus session
    PauseSession,
    /// Resume a paused session
    ResumeSession,
    /// Get current session status
    GetStatus,
    /// Ping the daemon to check if it's alive
    Ping,
}

/// Responses sent from daemon to CLI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// Session status information
    SessionStatus {
        /// Whether a session is currently active
        active: bool,
        /// Remaining time in seconds (0 if no session)
        remaining_seconds: u64,
        /// Current focus mode (None if no session)
        mode: Option<FocusMode>,
        /// Whether the session is paused
        paused: bool,
    },
    /// Generic success acknowledgment
    Ok,
    /// Error response with message
    Error { message: String },
    /// Pong response to ping
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_mode_serialization_roundtrip() {
        let modes = vec![
            FocusMode::Prompting,
            FocusMode::Review,
            FocusMode::Architecture,
            FocusMode::Custom("deep-work".to_string()),
        ];

        for mode in modes {
            let bytes = bincode::serialize(&mode).unwrap();
            let decoded: FocusMode = bincode::deserialize(&bytes).unwrap();
            assert_eq!(mode, decoded);
        }
    }

    #[test]
    fn request_start_session_serialization() {
        let request = Request::StartSession {
            duration: Some(25),
            mode: Some(FocusMode::Prompting),
        };

        let bytes = bincode::serialize(&request).unwrap();
        let decoded: Request = bincode::deserialize(&bytes).unwrap();

        assert_eq!(request, decoded);
    }

    #[test]
    fn request_start_session_with_defaults() {
        let request = Request::StartSession {
            duration: None,
            mode: None,
        };

        let bytes = bincode::serialize(&request).unwrap();
        let decoded: Request = bincode::deserialize(&bytes).unwrap();

        assert_eq!(request, decoded);
    }

    #[test]
    fn request_variants_serialization() {
        let requests = vec![
            Request::StopSession,
            Request::PauseSession,
            Request::ResumeSession,
            Request::GetStatus,
            Request::Ping,
        ];

        for request in requests {
            let bytes = bincode::serialize(&request).unwrap();
            let decoded: Request = bincode::deserialize(&bytes).unwrap();
            assert_eq!(request, decoded);
        }
    }

    #[test]
    fn response_session_status_serialization() {
        let response = Response::SessionStatus {
            active: true,
            remaining_seconds: 1500,
            mode: Some(FocusMode::Architecture),
            paused: false,
        };

        let bytes = bincode::serialize(&response).unwrap();
        let decoded: Response = bincode::deserialize(&bytes).unwrap();

        assert_eq!(response, decoded);
    }

    #[test]
    fn response_variants_serialization() {
        let responses = vec![
            Response::Ok,
            Response::Error {
                message: "Session déjà active".to_string(),
            },
            Response::Pong,
        ];

        for response in responses {
            let bytes = bincode::serialize(&response).unwrap();
            let decoded: Response = bincode::deserialize(&bytes).unwrap();
            assert_eq!(response, decoded);
        }
    }
}
