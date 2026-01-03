use flux_protocol::{Request, Response};
use interprocess::local_socket::{tokio::prelude::*, GenericFilePath};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let uid = unsafe { libc::getuid() };
    let socket_path = format!("/run/user/{}/flux.sock", uid);

    println!("Connexion à {}...", socket_path);

    let mut stream = interprocess::local_socket::tokio::Stream::connect(
        socket_path.to_fs_name::<GenericFilePath>()?,
    )
    .await?;

    let request = Request::Ping;
    let payload = bincode::serialize(&request)?;
    let length = (payload.len() as u32).to_le_bytes();

    stream.write_all(&length).await?;
    stream.write_all(&payload).await?;
    stream.flush().await?;

    println!("→ Envoyé: {:?}", request);

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let response_len = u32::from_le_bytes(len_buf) as usize;

    let mut response_buf = vec![0u8; response_len];
    stream.read_exact(&mut response_buf).await?;

    let response: Response = bincode::deserialize(&response_buf)?;
    println!("← Reçu: {:?}", response);

    match response {
        Response::Pong => println!("✓ Le daemon répond !"),
        other => println!("Réponse inattendue: {:?}", other),
    }

    Ok(())
}
