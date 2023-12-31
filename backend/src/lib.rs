mod configuration;
mod dependencies;
mod event_dispatcher;
pub mod http;
mod runtime;
mod services_container;
pub mod thoughts;

pub type StdError = anyhow::Error;
pub type StdResult<T> = anyhow::Result<T>;

pub use configuration::{ConfigurationBuilder, ConfigurationFileParser};
pub use dependencies::*;
pub use event_dispatcher::*;
pub use runtime::*;
pub use services_container::*;
