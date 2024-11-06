use anyhow::Result;
use tokio::{io, net::TcpListener};

pub async fn serve(listen_addr: &str) -> Result<()> {
    let listener = TcpListener::bind(&cli.ip_addr)
        .await
        .expect("failed to bind listener");
    log::info!(target: "logtail", "listening on {:?}", listener.local_addr()?);

    while let Ok((stream, addr)) = listener.accept().await {
        log::info!(target: "logtail", "accept a client: {:?}", addr);
        stream.readable().await?;
        io::copy(&mut stream, &mut io::stdout()).await?;
    }

    Err(anyhow::anyhow!("logtail listener exited"))
}
