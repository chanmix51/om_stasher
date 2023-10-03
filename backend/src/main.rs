use clap::Parser;

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

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
