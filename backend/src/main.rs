use anyhow::anyhow;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use flat_config::pool::SimpleFlatPool;

use backend::{ConfigurationBuilder, DependenciesBuilder, LoggerServiceRuntime, StdResult};

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

    /// HTTP server port (default 80)
    #[arg(long, default_value = "80", env = "OMSTASHER_BACKEND_HTTP_PORT")]
    http_port: u16,

    /// Postgres DSN
    #[arg(long, env = "OMSTASHER_DATABASE_DSN")]
    database_dsn: String,

    /// Verbose mode (-q, -v, -vv, -vvv, etc)
    #[command(flatten)]
    verbose: Verbosity,
}

impl CommandLineParameters {
    /// This function converts parameters values to FlatPool Values.
    pub fn to_flat_pool(self) -> SimpleFlatPool {
        let mut flat_pool = SimpleFlatPool::default();
        let tcp_port = self.http_port as isize;

        flat_pool
            .add("database_dsn", self.database_dsn.as_str().into())
            .add("http_address", self.http_address.as_str().into())
            .add("http_port", tcp_port.into());

        flat_pool
    }
}

#[tokio::main]
async fn main() -> StdResult<()> {
    let parameters = CommandLineParameters::parse();

    // logger initialization
    stderrlog::new()
        .module(module_path!())
        .quiet(parameters.verbose.is_silent())
        .verbosity(parameters.verbose.log_level_filter())
        .init()?;

    // Do not forget to update `to_flat_pool` function when new command line parameters are added.
    let dependencies =
        DependenciesBuilder::new(ConfigurationBuilder::new(parameters.to_flat_pool()));

    // HTTP server runtime initialization
    let http_service_runtime = dependencies.build_http_runtime().await?;

    // ThoughtService runtime initialization
    let thought_service_runtime = dependencies.build_thought_runtime().await?;

    // Logger service runtime.
    let logger_service_runtime = dependencies.build_logger_service_runtime().await?;

    // Launch all runtimes
    let runtime_result = tokio::select! {
        res = http_service_runtime.run() => res.map_err(|e| anyhow!(e)),
        res = thought_service_runtime.run() => res,
        res = logger_service_runtime.run() => res,
    };

    println!("Quitting…");

    runtime_result
}
