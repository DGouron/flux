use crate::actors::TimerHandle;
use anyhow::{Context, Result};
use flux_protocol::{FocusMode, Request, Response};
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath, ListenerOptions,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, instrument};

const DEFAULT_DURATION_MINUTES: u64 = 25;
const DEFAULT_CHECK_IN_MINUTES: u64 = 25;

pub struct Server {
    socket_path: PathBuf,
    timer_handle: TimerHandle,
}

impl Server {
    pub fn new(timer_handle: TimerHandle) -> Result<Self> {
        let uid = unsafe { libc::getuid() };
        let socket_path = PathBuf::from(format!("/run/user/{}/flux.sock", uid));
        Ok(Self {
            socket_path,
            timer_handle,
        })
    }

    fn cleanup_stale_socket(&self) -> Result<()> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .context("impossible de supprimer l'ancien socket")?;
            debug!("removed stale socket file");
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn run(&self, mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        self.cleanup_stale_socket()?;

        let listener = ListenerOptions::new()
            .name(self.socket_path.as_os_str().to_fs_name::<GenericFilePath>()?)
            .create_tokio()?;

        info!(path = %self.socket_path.display(), "server listening");

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok(stream) => {
                            let timer_handle = self.timer_handle.clone();
                            tokio::spawn(async move {
                                if let Err(error) = handle_connection(stream, timer_handle).await {
                                    error!(%error, "connection handler failed");
                                }
                            });
                        }
                        Err(error) => {
                            error!(%error, "failed to accept connection");
                        }
                    }
                }
                _ = shutdown.recv() => {
                    info!("shutdown signal received");
                    break;
                }
            }
        }

        self.cleanup_socket();
        Ok(())
    }

    fn cleanup_socket(&self) {
        if let Err(error) = std::fs::remove_file(&self.socket_path) {
            debug!(%error, "socket file already removed");
        } else {
            debug!("socket file cleaned up");
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.cleanup_socket();
    }
}

async fn handle_connection(mut stream: Stream, timer_handle: TimerHandle) -> Result<()> {
    debug!("new connection accepted");

    let mut length_buffer = [0u8; 4];
    stream.read_exact(&mut length_buffer).await?;
    let length = u32::from_le_bytes(length_buffer) as usize;

    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload).await?;

    let request: Request =
        bincode::deserialize(&payload).context("failed to deserialize request")?;

    debug!(?request, "received request");

    let response = handle_request(request, &timer_handle).await;

    debug!(?response, "sending response");

    let response_bytes = bincode::serialize(&response)?;
    let response_length = (response_bytes.len() as u32).to_le_bytes();

    stream.write_all(&response_length).await?;
    stream.write_all(&response_bytes).await?;
    stream.flush().await?;

    Ok(())
}

async fn handle_request(request: Request, timer_handle: &TimerHandle) -> Response {
    match request {
        Request::Ping => Response::Pong,

        Request::GetStatus => {
            if let Some(status) = timer_handle.get_status().await {
                Response::SessionStatus {
                    active: status.active,
                    remaining_seconds: status.remaining.as_secs(),
                    mode: status.mode,
                    paused: status.paused,
                }
            } else {
                Response::Error {
                    message: "impossible de récupérer le statut".to_string(),
                }
            }
        }

        Request::StartSession { duration, mode } => {
            let duration_minutes = duration.unwrap_or(DEFAULT_DURATION_MINUTES);
            let focus_mode = mode.unwrap_or(FocusMode::Prompting);

            if timer_handle
                .start(
                    Duration::from_secs(duration_minutes * 60),
                    focus_mode,
                    Duration::from_secs(DEFAULT_CHECK_IN_MINUTES * 60),
                )
                .await
                .is_ok()
            {
                Response::Ok
            } else {
                Response::Error {
                    message: "impossible de démarrer la session".to_string(),
                }
            }
        }

        Request::StopSession => {
            if timer_handle.stop().await.is_ok() {
                Response::Ok
            } else {
                Response::Error {
                    message: "impossible d'arrêter la session".to_string(),
                }
            }
        }

        Request::PauseSession => {
            if timer_handle.pause().await.is_ok() {
                Response::Ok
            } else {
                Response::Error {
                    message: "impossible de mettre en pause".to_string(),
                }
            }
        }

        Request::ResumeSession => {
            if timer_handle.resume().await.is_ok() {
                Response::Ok
            } else {
                Response::Error {
                    message: "impossible de reprendre la session".to_string(),
                }
            }
        }
    }
}
