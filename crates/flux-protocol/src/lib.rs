//! Flux protocol definitions for CLI-daemon communication

use serde::{Deserialize, Serialize};

/// Requests sent from CLI to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    /// Request daemon status
    Status,
    /// Request daemon to stop
    Stop,
    /// Ping the daemon
    Ping,
}

/// Responses sent from daemon to CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Status response with daemon state
    Status { running: bool, pid: u32 },
    /// Acknowledgment response
    Ok,
    /// Error response with message
    Error { message: String },
    /// Pong response to ping
    Pong,
}
