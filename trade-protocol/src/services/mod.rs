use std::sync::Arc;

use quinn::Connection;
use tarpc::context;

#[tarpc::service]
pub trait FinancialService {
    async fn hello(name: String) -> String;
    async fn send_heartbeat();
    async fn request_allocation() -> String;
}

#[async_trait::async_trait]
pub trait FinancialServiceHandler {
    async fn hello(self: Arc<Self>, name: String) -> String;
    async fn send_heartbeat(self: Arc<Self>);
    async fn request_allocation(self: Arc<Self>) -> String;
}

#[derive(Clone)]
pub struct FinancialServer<H: FinancialServiceHandler + Send + 'static + std::marker::Sync>(
    pub Arc<Connection>,
    pub Arc<H>,
);

#[tarpc::server]
impl<H: FinancialServiceHandler + Send + 'static + std::marker::Sync> FinancialService
    for FinancialServer<H>
{
    async fn hello(self, _: context::Context, name: String) -> String {
        self.1.hello(name).await
    }
    async fn send_heartbeat(self, _: context::Context) {
        self.1.send_heartbeat().await
    }
    async fn request_allocation(self, _: context::Context) -> String {
        self.1.request_allocation().await
    }
}
