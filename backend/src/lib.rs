mod configuration;
mod dependencies;
pub mod http;
pub mod thoughts;

pub type StdError = anyhow::Error;
pub type StdResult<T> = anyhow::Result<T>;

pub use dependencies::*;
