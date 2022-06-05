use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum TracingMode {
    Simple,
    Batch,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct NodeConfig {
    pub environment: NodeEnvironment,
    pub host_address: String,
    pub host_port: u16,
    pub local_address: String,
    pub local_port: u16,
    pub tracing_mode: Option<TracingMode>,
    pub jaeger_agent_endpoint: Option<String>,
    pub jaeger_collector_endpoint: Option<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            environment: NodeEnvironment::Development,
            host_address: "127.0.0.1".to_string(),
            host_port: 4001,
            local_address: "0.0.0.0".to_string(),
            local_port: 4002,
            tracing_mode: None,
            jaeger_agent_endpoint: None,
            jaeger_collector_endpoint: None,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum NodeEnvironment {
    Development,
    Staging,
    Production,
}
