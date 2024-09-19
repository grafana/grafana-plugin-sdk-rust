use std::collections::HashMap;

/// The error returned when Grafana does not provide a requested configuration option.
#[derive(Debug, Clone, thiserror::Error)]
#[error("key {key} not found in Grafana config; a more recent version of Grafana may be required")]
pub struct ConfigError {
    key: String,
}

impl ConfigError {
    fn from_key(key: String) -> Self {
        Self { key }
    }
}

type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// Configuration passed to the plugin from Grafana.
#[derive(Clone)]
pub struct GrafanaConfig {
    config: HashMap<String, String>,
}

impl GrafanaConfig {
    const APP_URL: &'static str = "GF_APP_URL";
    const APP_CLIENT_SECRET: &'static str = "GF_PLUGIN_APP_CLIENT_SECRET";
    // const CONCURRENT_QUERY_COUNT: &'static str = "GF_CONCURRENT_QUERY_COUNT";
    // const USER_FACING_DEFAULT_ERROR: &'static str = "GF_USER_FACING_DEFAULT_ERROR";
    // const SQL_ROW_LIMIT: &'static str = "GF_SQL_ROW_LIMIT";
    // const SQL_MAX_OPEN_CONNS_DEFAULT: &'static str = "GF_SQL_MAX_OPEN_CONNS_DEFAULT";
    // const SQL_MAX_IDLE_CONNS_DEFAULT: &'static str = "GF_SQL_MAX_IDLE_CONNS_DEFAULT";
    // const SQL_MAX_CONN_LIFETIME_SECONDS_DEFAULT: &'static str =
    //     "GF_SQL_MAX_CONN_LIFETIME_SECONDS_DEFAULT";
    // const RESPONSE_LIMIT: &'static str = "GF_RESPONSE_LIMIT";

    pub(crate) fn new(config: HashMap<String, String>) -> Self {
        Self { config }
    }

    /// Get the value of a configuration option, if it was provided by Grafana.
    fn get(&self, key: &str) -> ConfigResult<&String> {
        self.config
            .get(key)
            .ok_or_else(|| ConfigError::from_key(key.to_string()))
    }

    /// Return the URL of the Grafana instance.
    pub fn app_url(&self) -> ConfigResult<&String> {
        self.get(Self::APP_URL)
    }

    /// Return the client secret for the app plugin's service account, if set.
    ///
    /// Plugins can request a service account be created by Grafana at startup
    /// time by using the `iam` field of their `plugin.json` file. This method
    /// will then return the client secret for that service account, which can
    /// be used to authenticate with the Grafana API.
    ///
    /// See [this example plugin][example] for a full example of how to use this.
    ///
    /// [example]: https://github.com/grafana/grafana-plugin-examples/tree/main/examples/app-with-service-account
    pub fn plugin_app_client_secret(&self) -> ConfigResult<&String> {
        self.get(Self::APP_CLIENT_SECRET)
    }
}

impl std::fmt::Debug for GrafanaConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrafanaConfig").finish()
    }
}
