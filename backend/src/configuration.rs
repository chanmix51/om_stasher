use std::{
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::anyhow;
use flat_config::{
    pool::{LayeredFlatPool, SimpleFlatPool},
    ConfigBuilder, ConfigError, FlatValue,
};
use log::debug;
use tokio::sync::OnceCell;
use toml::{Table, Value};

use crate::StdResult;

const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";

type DynFlatPool = LayeredFlatPool;

// Configuration file management
/// Configuration file parser
pub struct ConfigurationFileParser {
    filepath: PathBuf,
}

impl ConfigurationFileParser {
    /// ## Configuration file constructor.
    ///
    /// Either a filename is given, in this case, if the file does not exist then this is an error,
    /// otherwise, the DEFAULT_CONFIG_FILE_NAME is tried. If it does not exist then Ok(None) is
    /// returned.
    pub fn new(filepath: Option<&PathBuf>) -> StdResult<Option<Self>> {
        if let Some(config_file) = filepath {
            debug!(
                " → got configuration file path '{}' from environment",
                config_file.display()
            );

            if config_file.exists() {
                Ok(Some(Self {
                    filepath: config_file.to_owned(),
                }))
            } else {
                Err(anyhow!(
                    "Configuration file '{}' does not exist",
                    config_file.display()
                ))
            }
        } else {
            let config_file = Path::new(DEFAULT_CONFIG_FILE_NAME).to_path_buf();
            if config_file.exists() {
                debug!(
                    " → found default configuration file path '{}'",
                    config_file.display()
                );

                Ok(Some(Self {
                    filepath: config_file.to_owned(),
                }))
            } else {
                Ok(None)
            }
        }
    }

    fn read_file(&self) -> StdResult<String> {
        let mut config_file = File::open(&self.filepath)?;
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;

        Ok(content)
    }

    fn parse_value(&self, value: &Value) -> StdResult<FlatValue> {
        let flat_value = match value {
            Value::String(v) => FlatValue::Text(v.clone()),
            Value::Integer(i) => FlatValue::Integer((*i).try_into()?),
            Value::Boolean(t) => FlatValue::Boolean(*t),
            _ => {
                return Err(anyhow!(
                    "could not parse toml value '{value:?}' as a flat_config_value."
                ))
            }
        };

        Ok(flat_value)
    }

    /// return the flat config representation of this .toml file
    pub fn parse(&self) -> StdResult<SimpleFlatPool> {
        let mut flat_pool = SimpleFlatPool::default();

        for (name, value) in self.read_file()?.parse::<Table>()? {
            flat_pool.add(name.as_str(), self.parse_value(&value)?);
        }

        Ok(flat_pool)
    }
}

/// Global configuration builder
pub struct ConfigurationBuilder {
    flat_pool: DynFlatPool,
    http_config: OnceCell<Arc<crate::http::BackendHttpConfig>>,
    thought_config: OnceCell<Arc<crate::thoughts::ThoughtServiceConfig>>,
}

impl ConfigurationBuilder {
    pub fn new(flat_pool: DynFlatPool) -> Self {
        Self {
            flat_pool,
            http_config: OnceCell::new(),
            thought_config: OnceCell::new(),
        }
    }

    async fn build_http_config(&self) -> Result<Arc<crate::http::BackendHttpConfig>, ConfigError> {
        let config = crate::http::BackendHttpConfigBuilder {}.build(&self.flat_pool)?;

        Ok(Arc::new(config))
    }

    pub async fn get_http_config(
        &self,
    ) -> Result<Arc<crate::http::BackendHttpConfig>, ConfigError> {
        let init = self.build_http_config();

        self.http_config
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }

    async fn build_thought_config(
        &self,
    ) -> Result<Arc<crate::thoughts::ThoughtServiceConfig>, ConfigError> {
        let config = crate::thoughts::ThoughtServiceConfigBuilder {}.build(&self.flat_pool)?;

        Ok(Arc::new(config))
    }

    pub async fn get_thought_config(
        &self,
    ) -> Result<Arc<crate::thoughts::ThoughtServiceConfig>, ConfigError> {
        let init = self.build_thought_config();

        self.thought_config
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }
}
