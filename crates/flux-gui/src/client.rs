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
        let socket_path = Self::default_socket_path();
        Self {
            socket_path,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    #[cfg(unix)]
    fn default_socket_path() -> PathBuf {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/run/user/{}/flux.sock", uid))
    }

    #[cfg(windows)]
    fn default_socket_path() -> PathBuf {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(format!(r"{}\flux\flux.sock", local_app_data))
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
