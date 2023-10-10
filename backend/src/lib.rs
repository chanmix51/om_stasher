mod configuration;
mod dependencies;
pub mod http;
mod services_container;
pub mod thoughts;

pub type StdError = anyhow::Error;
pub type StdResult<T> = anyhow::Result<T>;

pub use dependencies::*;
pub use services_container::*;
