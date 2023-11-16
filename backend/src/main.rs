use std::path::PathBuf;

use anyhow::anyhow;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use flat_config::{
    pool::{FlatPool, LayeredFlatPool, SimpleFlatPool},
    TryUnwrap,
};
use futures::stream::StreamExt;
use log::{debug, error, info, trace, warn};
use signal_hook::consts::*;
use signal_hook_tokio::Signals;

use backend::{
    ConfigurationBuilder, ConfigurationFileParser, DependenciesBuilder, EventDispatcherLoop,
    StdResult,
};

/// Possible command line options and arguments
#[derive(Debug, Parser)]
pub struct CommandLineParameters {
    /// Configuration file location if any
    #[arg(long, short, env = "OMSTASHER_CONFIG_FILE")]
    config_file: Option<PathBuf>,

    /// HTTP server bind address
    #[arg(long, env = "OMSTASHER_BACKEND_HTTP_ADDRESS")]
    http_address: Option<String>,

    /// HTTP server port
    #[arg(long, env = "OMSTASHER_BACKEND_HTTP_PORT")]
    http_port: Option<u16>,

    /// Postgres DSN
    #[arg(long, env = "OMSTASHER_DATABASE_DSN")]
    database_dsn: Option<String>,

    /// Verbose mode (-q, -v, -vv, -vvv, etc)
    #[command(flatten)]
    verbose: Verbosity,
}

impl CommandLineParameters {
    /// This function converts parameters values to FlatPool Values.
    pub fn to_flat_pool(self) -> SimpleFlatPool {
        let mut flat_pool = SimpleFlatPool::default();

        if let Some(tcp_port) = self.http_port {
            flat_pool.add("http_port", (tcp_port as isize).into());
        }

        if let Some(database_dsn) = &self.database_dsn {
            flat_pool.add("database_dsn", database_dsn.as_str().into());
        }

        if let Some(http_address) = &self.http_address {
            flat_pool.add("http_address", http_address.as_str().into());
        }

        flat_pool
    }
}

/// Set the application default parameters
/// Theses parameters will be assumed to always be present in the [ConfigurationBuilder]
fn default_parameters() -> SimpleFlatPool {
    // compilation constants
    // code parsed at compilation time

    /// configuration file that will be searched for if no other options are provided.
    const DEFAULT_CONFIG_FILE: &str = match option_env!("OMSTASHER_BACKEND_DEFAULT_CONFIG_FILE") {
        Some(s) => s,
        None => "backend.config.toml",
    };

    /// default TCP port for the HTTP server
    const DEFAULT_HTTP_PORT: &str = match option_env!("OMSTASHER_BACKEND_DEFAULT_HTTP_PORT") {
        Some(s) => s,
        None => "80",
    };

    /// default IP address the HTTP server will bind to
    const DEFAULT_HTTP_ADDRESS: &str = match option_env!("OMSTASHER_BACKEND_DEFAULT_HTTP_ADDRESS") {
        Some(s) => s,
        None => "127.0.0.1",
    };

    let mut flat_pool = SimpleFlatPool::default();
    flat_pool
        .add("http_address", DEFAULT_HTTP_ADDRESS.into())
        .add(
            "http_port",
            DEFAULT_HTTP_PORT.parse::<isize>().unwrap().into(),
        )
        // the name 'default_config_file' is therefore reserved for our usage
        // and will be EXPECTED to be present at runtime.
        .add("default_config_file", DEFAULT_CONFIG_FILE.into());

    flat_pool
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
    info!(
        "starting OMStasher backend version {}",
        env!("CARGO_PKG_VERSION")
    );
    debug!("Command line parameters: {parameters:?}");

    trace!("manage configuration");
    let mut flat_pools: Vec<Box<dyn FlatPool>> = Vec::new();
    // 1 - Add default parameters first
    flat_pools.push(Box::new(default_parameters()));

    if let Some(config_file) = ConfigurationFileParser::new(
        parameters.config_file.as_ref(),
        default_parameters()
            .require("default_config_file")
            .map(|v| -> String { v.try_unwrap().unwrap() })
            .expect("No default configuration file compiled in!")
            .as_ref(),
    )? {
        // 2 - Add configuration file parameters
        flat_pools.push(Box::new(config_file.parse()?));
    }

    // 3 - Add CLI and ENV parameters
    flat_pools.push(Box::new(parameters.to_flat_pool()));
    debug!("configuration settings: {flat_pools:?}");

    trace!("initialize dependencies builder");
    let dependencies =
        DependenciesBuilder::new(ConfigurationBuilder::new(LayeredFlatPool::new(flat_pools)));

    trace!("HTTP server runtime initialization");
    let http_runtime = dependencies.build_http_runtime().await?;

    trace!("thought runtime initialization");
    let thought_runtime = dependencies.build_thought_runtime().await?;

    trace!("logger runtime initialization");
    let logger_runtime = dependencies.build_logger_runtime().await?;

    trace!("create event dispatcher loop");
    let dispatcher_loop = EventDispatcherLoop::new(dependencies.get_event_dispatcher().await?);

    trace!("create signal handler and hook");
    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?;
    let signal_handler = signals.handle();

    // The dependencies builder is dropped in order to remove all Arc instances in it.
    trace!("dropping dependencies");
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
