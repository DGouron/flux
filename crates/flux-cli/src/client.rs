use flux_protocol::{Request, Response};
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath,
};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("le daemon n'est pas en cours d'exécution")]
    DaemonNotRunning,
    #[error("timeout lors de la communication avec le daemon")]
    Timeout,
    #[error("erreur de connexion: {0}")]
    Connection(String),
    #[error("erreur de sérialisation: {0}")]
    Serialization(String),
    #[error("erreur d'entrée/sortie: {0}")]
    InputOutput(String),
}

pub struct DaemonClient {
    socket_path: PathBuf,
    timeout: Duration,
}

impl DaemonClient {
    pub fn new() -> Self {
        let uid = unsafe { libc::getuid() };
        let socket_path = PathBuf::from(format!("/run/user/{}/flux.sock", uid));
        Self {
            socket_path,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn send(&self, request: Request) -> Result<Response, ClientError> {
        let stream = self.connect().await?;
        self.send_request(stream, request).await
    }

    async fn connect(&self) -> Result<Stream, ClientError> {
        let connect_future = Stream::connect(
            self.socket_path
                .as_os_str()
                .to_fs_name::<GenericFilePath>()
                .map_err(|error| ClientError::Connection(error.to_string()))?,
        );

        match timeout(self.timeout, connect_future).await {
            Ok(Ok(stream)) => Ok(stream),
            Ok(Err(_)) => Err(ClientError::DaemonNotRunning),
            Err(_) => Err(ClientError::Timeout),
        }
    }

    async fn send_request(
        &self,
        mut stream: Stream,
        request: Request,
    ) -> Result<Response, ClientError> {
        let request_bytes = bincode::serialize(&request)
            .map_err(|error| ClientError::Serialization(error.to_string()))?;

        let request_length = (request_bytes.len() as u32).to_le_bytes();

        let write_future = async {
            stream.write_all(&request_length).await?;
            stream.write_all(&request_bytes).await?;
            stream.flush().await?;
            Ok::<_, std::io::Error>(())
        };

        timeout(self.timeout, write_future)
            .await
            .map_err(|_| ClientError::Timeout)?
            .map_err(|error| ClientError::InputOutput(error.to_string()))?;

        let read_future = async {
            let mut length_buffer = [0u8; 4];
            stream.read_exact(&mut length_buffer).await?;
            let length = u32::from_le_bytes(length_buffer) as usize;

            let mut payload = vec![0u8; length];
            stream.read_exact(&mut payload).await?;
            Ok::<_, std::io::Error>(payload)
        };

        let response_bytes = timeout(self.timeout, read_future)
            .await
            .map_err(|_| ClientError::Timeout)?
            .map_err(|error| ClientError::InputOutput(error.to_string()))?;

        bincode::deserialize(&response_bytes)
            .map_err(|error| ClientError::Serialization(error.to_string()))
    }
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_protocol::Request;
    use interprocess::local_socket::{GenericFilePath, ListenerOptions};
    use std::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn test_socket_path() -> PathBuf {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/flux-test-{}.sock", uid))
    }

    fn cleanup_socket(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn client_creates_with_default_timeout() {
        let client = DaemonClient::new();
        assert_eq!(client.timeout, Duration::from_secs(5));
    }

    #[test]
    fn client_with_custom_timeout() {
        let client = DaemonClient::new().with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn send_returns_error_when_daemon_not_running() {
        let mut client = DaemonClient::new();
        client.socket_path = PathBuf::from("/tmp/flux-nonexistent-socket-12345.sock");

        let result = client.send(Request::Ping).await;

        assert!(
            matches!(result, Err(ClientError::DaemonNotRunning)),
            "expected DaemonNotRunning, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn send_ping_receives_pong() {
        let socket_path = test_socket_path();
        cleanup_socket(&socket_path);

        let server_path = socket_path.clone();
        let server_handle = tokio::spawn(async move {
            let listener = ListenerOptions::new()
                .name(
                    server_path
                        .as_os_str()
                        .to_fs_name::<GenericFilePath>()
                        .unwrap(),
                )
                .create_tokio()
                .unwrap();

            let mut stream = listener.accept().await.unwrap();

            let mut length_buffer = [0u8; 4];
            stream.read_exact(&mut length_buffer).await.unwrap();
            let length = u32::from_le_bytes(length_buffer) as usize;

            let mut payload = vec![0u8; length];
            stream.read_exact(&mut payload).await.unwrap();

            let request: Request = bincode::deserialize(&payload).unwrap();
            assert!(matches!(request, Request::Ping));

            let response = Response::Pong;
            let response_bytes = bincode::serialize(&response).unwrap();
            let response_length = (response_bytes.len() as u32).to_le_bytes();

            stream.write_all(&response_length).await.unwrap();
            stream.write_all(&response_bytes).await.unwrap();
            stream.flush().await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut client = DaemonClient::new();
        client.socket_path = socket_path.clone();

        let result = client.send(Request::Ping).await;

        assert!(matches!(result, Ok(Response::Pong)));

        server_handle.await.unwrap();
        cleanup_socket(&socket_path);
    }

    #[tokio::test]
    async fn send_get_status_receives_session_status() {
        let socket_path = test_socket_path();
        let unique_path = PathBuf::from(format!("{}-status", socket_path.display()));
        cleanup_socket(&unique_path);

        let server_path = unique_path.clone();
        let server_handle = tokio::spawn(async move {
            let listener = ListenerOptions::new()
                .name(
                    server_path
                        .as_os_str()
                        .to_fs_name::<GenericFilePath>()
                        .unwrap(),
                )
                .create_tokio()
                .unwrap();

            let mut stream = listener.accept().await.unwrap();

            let mut length_buffer = [0u8; 4];
            stream.read_exact(&mut length_buffer).await.unwrap();
            let length = u32::from_le_bytes(length_buffer) as usize;

            let mut payload = vec![0u8; length];
            stream.read_exact(&mut payload).await.unwrap();

            let request: Request = bincode::deserialize(&payload).unwrap();
            assert!(matches!(request, Request::GetStatus));

            let response = Response::SessionStatus {
                active: true,
                remaining_seconds: 1500,
                mode: Some(flux_protocol::FocusMode::Prompting),
                paused: false,
            };
            let response_bytes = bincode::serialize(&response).unwrap();
            let response_length = (response_bytes.len() as u32).to_le_bytes();

            stream.write_all(&response_length).await.unwrap();
            stream.write_all(&response_bytes).await.unwrap();
            stream.flush().await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut client = DaemonClient::new();
        client.socket_path = unique_path.clone();

        let result = client.send(Request::GetStatus).await;

        match result {
            Ok(Response::SessionStatus {
                active,
                remaining_seconds,
                paused,
                ..
            }) => {
                assert!(active);
                assert_eq!(remaining_seconds, 1500);
                assert!(!paused);
            }
            _ => panic!("expected SessionStatus response"),
        }

        server_handle.await.unwrap();
        cleanup_socket(&unique_path);
    }
}
