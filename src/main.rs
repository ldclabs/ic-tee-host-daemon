use anyhow::Result;
use clap::Parser;
use structured_logger::{async_json::new_writer, get_env_level, unix_ms, Builder};
use tokio::{io, net::TcpListener};

mod helper;
mod ip_to_vsock;
mod logtail;
mod vsock_to_ip_transparent;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// vsock address of the listener side, usually open to the other side of the transparent proxy (e.g. 3:1200)
    #[clap(long)]
    vsock_to_ip_transparent: String,

    /// ip address of the listener side (e.g. 0.0.0.0:4000)
    #[clap(short, long, value_parser)]
    ip_to_vsock_ip_addr: String,

    /// vsock address of the upstream side (e.g. 88:4000)
    #[clap(short, long, value_parser)]
    ip_to_vsock_addr: String,

    /// address to listen for debug logs (e.g. 127.0.0.1:9999)
    #[arg(long)]
    debug_log_addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    Builder::with_level(&get_env_level().to_string())
        .with_target_writer("*", new_writer(io::stdout()))
        .init();

    let serve_vsock_to_ip_transparent = async {
        let vsock_addr = helper::split_vsock(&cli.vsock_to_ip_transparent)?;
        vsock_to_ip_transparent::serve(vsock_addr).await?;
        Ok(())
    };

    let serve_ip_to_vsock = async {
        let vsock_addr = helper::split_vsock(&cli.ip_to_vsock_addr)?;
        ip_to_vsock::serve(&cli.ip_to_vsock_ip_addr, vsock_addr).await?;
        Ok(())
    };

    let serve_debug_log_addr = async {
        if let Some(ref addr) = cli.debug_log_addr {
            logtail::serve(addr).await?;
        }
        Ok(())
    };

    match tokio::try_join!(
        serve_vsock_to_ip_transparent,
        serve_ip_to_vsock,
        serve_debug_log_addr
    ) {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!(target: "server", "server error: {:?}", err);
            Err(err)
        }
    }
}
