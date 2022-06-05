use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum TracingMode {
    Simple,
    Batch,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct HostConfig {
    pub environment: HostEnvironment,
    pub cert_path: PathBuf,
    pub misc_path: PathBuf,
    pub cert_names: Vec<String>,
    pub host: String,
    pub port: u16,
    pub binance_api_key: Option<String>,
    pub binance_secret_key: Option<String>,
    pub tracing_mode: Option<TracingMode>,
    pub jaeger_agent_endpoint: Option<String>,
    pub jaeger_collector_endpoint: Option<String>,
    pub influx_url: Option<String>,
    pub influx_username: Option<String>,
    pub influx_password: Option<String>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            environment: HostEnvironment::Development,
            cert_path: PathBuf::from("./data/certs"),
            misc_path: PathBuf::from("./data/misc"),
            cert_names: vec!["localhost".to_string(), "host".to_string()],
            host: "0.0.0.0".to_string(),
            port: 4001,
            binance_api_key: None,
            binance_secret_key: None,
            tracing_mode: None,
            jaeger_agent_endpoint: None,
            jaeger_collector_endpoint: None,
            influx_url: None,
            influx_username: None,
            influx_password: None,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum HostEnvironment {
    Development,
    Staging,
    Production,
}
