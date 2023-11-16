use std::{fs::File, io::prelude::*, path::PathBuf, sync::Arc};

use anyhow::anyhow;
use flat_config::{
    pool::{LayeredFlatPool, SimpleFlatPool},
    ConfigBuilder, ConfigError, FlatValue,
};
use log::debug;
use tokio::sync::OnceCell;
use toml::{Table, Value};

use crate::StdResult;

type DynFlatPool = LayeredFlatPool;

// Configuration file management
/// Configuration file parser
pub struct ConfigurationFileParser {
    filepath: PathBuf,
}

impl ConfigurationFileParser {
    /// ## Configuration file constructor.
    ///
    /// If an environment filepath is provided, it WILL be used hence it will be an error if it
    /// does not exist. If no environment filepath is provided, the default one will be probed. If
    /// it does not exist, the function returns None, otherwise, the file is added to the
    /// configuration.
    pub fn new(env_filepath: Option<&PathBuf>, default_filename: &str) -> StdResult<Option<Self>> {
        if let Some(filepath) = env_filepath {
            let filename = filepath.display();
            debug!(" → got configuration file path '{filename}' from environment",);

            if filepath.exists() {
                Ok(Some(Self {
                    filepath: filepath.to_owned(),
                }))
            } else {
                Err(anyhow!("Configuration file '{filename}' does not exist",))
            }
        } else {
            let filepath = PathBuf::new().join(default_filename);
            if filepath.exists() {
                debug!(" → found default configuration file path '{default_filename}'",);

                Ok(Some(Self { filepath: filepath }))
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
