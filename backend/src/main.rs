use backend::{dependencies::Dependencies, StdResult};
use clap::Parser;
use flat_config::pool::SimpleFlatPool;

/// Possible command line options and arguments
#[derive(Debug, Parser)]
pub struct CommandLineParameters {
    /// HTTP server bind address (default 127.0.0.1)
    #[arg(
        long,
        default_value = "127.0.0.1",
        env = "OMSTASHER_BACKEND_HTTP_ADDRESS"
    )]
    http_address: String,

    /// HTTP serve port (default 80)
    #[arg(long, default_value = "80", env = "OMSTASHER_BACKEND_HTTP_PORT")]
    http_port: u16,
}

impl CommandLineParameters {
    pub fn to_flat_pool(self) -> SimpleFlatPool {
        let mut flat_pool = SimpleFlatPool::default();
        let tcp_port = self.http_port as isize;

        flat_pool
            .add("http_address", self.http_address.as_str().into())
            .add("http_port", tcp_port.into());

        flat_pool
    }
}

#[tokio::main]
async fn main() -> StdResult<()> {
    let params = CommandLineParameters::parse().to_flat_pool();
    let mut dependencies = Dependencies::new(params);
    let http_service = dependencies.build_http_service().await?;
    let http_service_handle = tokio::spawn(async move { http_service.run().await });

    tokio::select! {
    _ = http_service_handle => ()
    }
    println!("Quitting…");

    Ok(())
}
