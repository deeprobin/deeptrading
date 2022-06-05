use std::sync::Arc;

use async_trait::async_trait;
use binance::{api::Binance, general::General};
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::host::Host;

use super::Service;

pub struct BinanceService {
    api_key: Option<String>,
    secret_key: Option<String>,
    // Symbol, Node Id
    pub symbol_node_map: Arc<Mutex<Vec<(String, Option<usize>)>>>,
}

impl BinanceService {
    fn get_binance<T: Binance + Sized>(self: Arc<Self>) -> T {
        let this = Arc::clone(&self);
        T::new(this.api_key.clone(), this.secret_key.clone())
    }
}

#[async_trait]
impl Service for BinanceService {
    // api_key, secret_key
    type Params = (Option<String>, Option<String>);
    async fn try_init(
        _host: Arc<Host>,
        params: Self::Params,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(BinanceService {
            api_key: params.0,
            secret_key: params.1,
            symbol_node_map: Arc::new(Mutex::new(vec![])),
        }))
    }
    async fn run(
        self: Arc<Self>,
        _shutdown_recv: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let this = Arc::clone(&self);
        let general_api = this.get_binance::<General>();
        match general_api.ping().await {
            Ok(res) => info!("Connectivity to Binance API established ({})", res),
            Err(err) => {
                error!("Cannot connect to Binance API");
                return Err(Box::new(err));
            }
        }

        let exchange_info = general_api
            .exchange_info()
            .await
            .expect("Failed to fetch exchange info");

        let this = Arc::clone(&self);
        let mut symbol_node_map = this.symbol_node_map.lock().await;
        for symbol in exchange_info.clone().symbols {
            symbol_node_map.push((symbol.symbol, None));
        }
        info!("Loaded {} symbols", exchange_info.symbols.len());

        let this = Arc::clone(&self);
        let _market = this.get_binance::<binance::market::Market>();

        let this = Arc::clone(&self);
        let _margin = this.get_binance::<binance::margin::Margin>();

        loop {}
    }
}
