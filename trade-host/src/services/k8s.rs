use std::sync::Arc;

use async_trait::async_trait;
use kube::Client;
use tokio::sync::broadcast::Receiver;
use tracing::{info, warn};

use crate::host::Host;

use super::Service;

pub struct KubernetesService {
    _kubernetes_client: Option<Client>,
}

#[async_trait]
impl Service for KubernetesService {
    // api_key, secret_key
    type Params = ();
    async fn try_init(
        _host: Arc<Host>,
        _params: Self::Params,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(match Client::try_default().await {
            Ok(client) => {
                info!("Established connection to Kubernetes-API");
                KubernetesService {
                    _kubernetes_client: Some(client),
                }
            }
            Err(err) => {
                warn!(
                    "Cannot initialize, perhaps you're not in a kubernetes environment: {}",
                    err
                );
                KubernetesService {
                    _kubernetes_client: None,
                }
            }
        }))
    }
    async fn run(
        self: Arc<Self>,
        _shutdown_recv: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {}
    }
}
