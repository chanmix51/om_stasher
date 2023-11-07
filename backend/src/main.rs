use anyhow::anyhow;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use flat_config::pool::SimpleFlatPool;
use futures::stream::StreamExt;
use log::{error, info, trace, warn};
use signal_hook::consts::*;
use signal_hook_tokio::Signals;

use backend::{
    ConfigurationBuilder, DependenciesBuilder, EventDispatcher, EventDispatcherLoop, StdResult,
};

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

/// OS signal handler (only Linux for now)
pub struct OsSignalHandler;

impl OsSignalHandler {
    pub async fn handle_signal(mut signals: Signals) {
        while let Some(signal) = signals.next().await {
            match signal {
                SIGTERM | SIGINT | SIGQUIT => {
                    warn!("Signal caught: {signal}");

                    break;
                }
                _ => unreachable!(),
            }
        }
    }
}

#[tokio::main]
async fn main() -> StdResult<()> {
    // Read parameters from command line and environment.
    let parameters = CommandLineParameters::parse();

    // logger initialization
    stderrlog::new()
        .module(module_path!())
        .quiet(parameters.verbose.is_silent())
        .verbosity(parameters.verbose.log_level_filter())
        .init()?;
    info!("OMStasher backend version {}", env!("CARGO_PKG_VERSION"));

    trace!("Initialize dependencies builder");
    // Do not forget to update `to_flat_pool` function when new command line parameters are added.
    let dependencies =
        DependenciesBuilder::new(ConfigurationBuilder::new(parameters.to_flat_pool()));

    trace!("HTTP server runtime initialization");
    let http_runtime = dependencies.build_http_runtime().await?;

    trace!("Thought runtime initialization");
    let thought_runtime = dependencies.build_thought_runtime().await?;

    trace!("Logger runtime initialization");
    let logger_runtime = dependencies.build_logger_runtime().await?;

    trace!("create event dispatcher loop");
    let dispatcher_loop = EventDispatcherLoop::new(dependencies.get_event_dispatcher().await?);

    trace!("create signal handler and hook");
    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?;
    let signal_handler = signals.handle();

    // The dependencies builder is dropped in order to remove all Arc instances in it.
    drop(dependencies);

    trace!("launch all runtimes…");
    let runtime_result = tokio::select! {
        res = http_runtime.run() => res.map_err(|e| anyhow!(e)),
        res = thought_runtime.run() => res,
        res = logger_runtime.run() => res,
        _ = OsSignalHandler::handle_signal(signals) => Ok(()),
        _ = dispatcher_loop.tickle() => Err(anyhow!("Event dispatcher has terminated!")),
    };

    trace!("close signal handler");
    signal_handler.close();

    match &runtime_result {
        Err(e) => error!("{e}"),
        Ok(_) => info!("…Finishing OK."),
    };

    runtime_result
}
