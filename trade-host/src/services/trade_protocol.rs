use crate::host::Host;
use async_trait::async_trait;
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::broadcast::Receiver;
use tracing::info;
use trade_protocol::{
    listener::{TradeListener, TradeListenerHandler},
    packets::AllocationResponsePacketData,
    services::FinancialServiceHandler,
};

use super::{binance::BinanceService, certificate_check::CertificateCheckService, Service};

pub struct TradeProtocolService {
    host: Arc<Host>,
    binance_service: Arc<BinanceService>,
    certificate_service: Arc<CertificateCheckService>,
}

#[async_trait]
impl Service for TradeProtocolService {
    type Params = (Arc<BinanceService>, Arc<CertificateCheckService>);
    async fn try_init(
        host: Arc<Host>,
        params: Self::Params,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(Self {
            host,
            binance_service: params.0,
            certificate_service: params.1,
        }))
    }
    async fn run(
        self: Arc<Self>,
        _shutdown_recv: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let this = Arc::clone(&self);
        let cert_handler = Arc::clone(&this.certificate_service);
        let certs = cert_handler.get_certs().await;
        let mut listener = TradeListener::new(certs.0, certs.1).expect("Failed to create listener");

        let address_value = format!("{}:{}", self.host.config.host, self.host.config.port);
        let address = SocketAddr::from_str(&address_value).expect("Failed to parse address");

        let listen_task = listener.listen(address, Arc::new(|_| Arc::new(FinancialServiceImpl)));
        tokio::join!(
            async move {
                listen_task.await.expect("Cannot listen on address");
            },
            async move {
                info!("Listening on {}", address_value);
            }
        );
        Ok(())
    }
}

struct FinancialServiceImpl;

#[async_trait::async_trait]
impl FinancialServiceHandler for FinancialServiceImpl {
    async fn hello(self: Arc<Self>, name: String) -> String {
        "Hello".to_string()
    }
    async fn send_heartbeat(self: Arc<Self>) {}
    async fn request_allocation(self: Arc<Self>) -> String {
        "AAPL".to_string()
    }
}

struct TradeProtocolHandler {
    service: Arc<TradeProtocolService>,
}

#[async_trait]
impl TradeListenerHandler for TradeProtocolHandler {
    async fn on_connect(
        &mut self,
        connection: Arc<quinn::Connection>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Established connection from {}",
            connection.remote_address()
        );
        Ok(())
    }
    async fn handle_heartbeat(
        &mut self,
        connection: Arc<quinn::Connection>,
        _packet: trade_protocol::packets::HeartbeatPacketData,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Heartbeat received from {}", connection.remote_address());
        Ok(())
    }
    async fn handle_allocation_request(
        &mut self,
        connection: Arc<quinn::Connection>,
    ) -> Result<AllocationResponsePacketData, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Allocation request received from {}",
            connection.remote_address()
        );
        let binance_service = Arc::clone(&self.service.binance_service);
        let mut symbol_node_map = binance_service.symbol_node_map.lock().await;

        // TODO: Add prioritising some symbols
        if let Some((symbol, node_id)) = symbol_node_map
            .iter_mut()
            .find(|(_, node_id)| node_id.is_none())
        {
            *node_id = Some(connection.stable_id());
            info!(
                "Allocated symbol {} to node {}",
                symbol,
                connection.stable_id()
            );
            Ok(AllocationResponsePacketData {
                symbol: Some(symbol.clone()),
            })
        } else {
            info!("Cannot allocate symbol to node {}", connection.stable_id());
            Ok(AllocationResponsePacketData { symbol: None })
        }
    }
}
