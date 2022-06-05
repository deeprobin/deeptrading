use std::{
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use config::NodeConfig;

use rustls::{client::ServerCertVerifier, ClientConfig, RootCertStore};
use tarpc::context;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info, trace};
use trade_protocol::client::TradeClient;

use crate::pyd::PythonDaemon;
pub mod config;
mod interface;
mod pyd;

pub enum NodeState {
    Initialization,
    InitialTraining,
    ActiveTrading,
}

pub enum NodeMode {
    MainNode,
    BackupNode,
}

pub struct Node {
    config: NodeConfig,
}

impl Node {
    pub fn new(config: NodeConfig) -> Arc<Self> {
        Arc::new(Node { config })
    }

    pub async fn run(self: Arc<Self>, mut shutdown_recv: UnboundedReceiver<()>) {
        let this = Arc::clone(&self);
        let mut client = this.create_client().await;

        let connection = client.connect().await.expect("Failed to create connection");
        for i in 0..100 {
            connection
                .client
                .send_heartbeat({
                    let mut ctx = context::current();
                    ctx.deadline += Duration::from_secs(60);
                    ctx
                })
                .await
                .expect("Failed to send heartbeat");
            info!("Request {} of {}", i, 100);
        }
        info!("Established connection to host");

        info!("Requesting allocation");
        connection
            .client
            .request_allocation({
                let mut ctx = context::current();
                ctx.deadline += Duration::from_secs(60);
                ctx
            })
            .await
            .expect("Failed to request allocation");

        let shutdown_task = tokio::task::spawn(async move {
            let result = shutdown_recv.recv().await;
            if result.is_some() {
                info!("Shutdown requested");
            } else {
                error!("Shutdown guard receiver closed");
            }
        });

        let python_daemon_task = tokio::spawn(PythonDaemon.run());
        tokio::select! {
            _ = shutdown_task => {
                info!("Shutdown complete");
            },
            _ = python_daemon_task => {
                error!("Python daemon ended prematurely");
            }
        }
    }

    async fn create_client(self: Arc<Self>) -> TradeClient {
        let host_address_value = format!("{}:{}", self.config.host_address, self.config.host_port);
        let host_address = host_address_value
            .to_socket_addrs()
            .expect("Failed to resolve address")
            .next()
            .expect("No host address provided");
        let local_address_value =
            format!("{}:{}", self.config.local_address, self.config.local_port);

        let local_address =
            SocketAddr::from_str(&local_address_value).expect("Failed to parse address");
        let mut tls_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth();

        // TODO: Only allow this in development and use in staging/production a root CA
        tls_config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoVerifier));

        info!(
            "Connecting to host {} with local QUIC end point {}",
            host_address_value, local_address_value
        );
        let client = TradeClient::new(local_address, host_address, "trade-node", tls_config)
            .await
            .expect("Failed to initialize trade client");

        client
    }
}

pub struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        trace!("Insecure certificate validation for {:?}", server_name);
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
