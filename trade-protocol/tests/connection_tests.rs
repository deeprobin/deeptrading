use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::future::AbortHandle;
use futures_util::future::Abortable;
use rcgen::generate_simple_self_signed;
use rustls::client::ServerCertVerifier;
use rustls::RootCertStore;
use tarpc::context;
use tracing::{info, trace};
use tracing_test::traced_test;
use trade_protocol::client::TradeClient;
use trade_protocol::listener::TradeListener;
use trade_protocol::listener::TradeListenerHandler;
use trade_protocol::packets::AllocationResponsePacketData;
use trade_protocol::services::FinancialServiceHandler;

#[tokio::test]
#[traced_test]
async fn connection_works() {
    let server_name = "test-server";
    let generated_cert = generate_simple_self_signed(vec![server_name.into()])
        .expect("Failed to generate certificate");
    info!("Generated cert");

    let mut cert_store = RootCertStore::empty();

    let key = rustls::PrivateKey(generated_cert.serialize_private_key_der());
    let cert = rustls::Certificate(
        generated_cert
            .serialize_der()
            .expect("Failed to serialize certificate"),
    );
    cert_store
        .add(&cert)
        .expect("Failed to add certificate to store");
    info!("Created cert store");

    let mut tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(cert_store)
        .with_no_client_auth();
    tls_config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoVerifier));

    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let server_ep = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4040);
    let mut listener = TradeListener::new(vec![cert], key).expect("Failed to create listener");
    let handler = Arc::new(Handler {
        heartbeat_received: AtomicBool::new(false),
    });
    let handler_clone = Arc::clone(&handler);
    let listener_task = Abortable::new(
        async move {
            listener
                .listen::<Handler, _>(
                    server_ep,
                    Arc::new(move |connection| Arc::clone(&handler_clone)),
                )
                .await
                .expect("Failed to run listener");
            info!("Listener ended");
        },
        abort_registration,
    );
    tokio::spawn(listener_task);
    info!("Listener started");

    let client_ep = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4041);
    let mut client = TradeClient::new(client_ep, server_ep, server_name, tls_config)
        .await
        .expect("Failed to create client");
    info!("Client created");

    // Wait until listener is ready
    std::thread::sleep(Duration::from_millis(500));

    let connection = client.connect().await.expect("Failed to create connection");
    connection
        .client
        .send_heartbeat(context::current())
        .await
        .expect("Failed to send heartbeat");
    info!("Sent heartbeat");

    // Wait until heartbeat receive
    std::thread::sleep(Duration::from_millis(5000));
    assert_eq!(
        true,
        handler
            .heartbeat_received
            .load(std::sync::atomic::Ordering::Relaxed)
    );

    client.close().await;
    abort_handle.abort();
}

pub struct Handler {
    heartbeat_received: AtomicBool,
}

#[async_trait]
impl FinancialServiceHandler for Handler {
    async fn hello(self: Arc<Self>, name: String) -> String {
        "Hello".to_string()
    }
    async fn send_heartbeat(self: Arc<Self>) {
        println!("Received heartbeat");
        self.heartbeat_received.store(true, Ordering::Relaxed)
    }
    async fn request_allocation(self: Arc<Self>) -> String {
        "AAPL".to_string()
    }
}

struct NoVerifier;

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
