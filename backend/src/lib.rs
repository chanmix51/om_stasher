mod configuration;
mod dependencies;
mod event;
pub mod http;
mod services_container;
pub mod thoughts;

pub type StdError = anyhow::Error;
pub type StdResult<T> = anyhow::Result<T>;

pub use configuration::ConfigurationBuilder;
pub use dependencies::*;
pub use event::*;
pub use services_container::*;
