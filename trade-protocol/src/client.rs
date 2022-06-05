use crate::services::FinancialServiceClient;
use crate::StreamFramer;
use quinn::ClientConfig;
use std::{net::SocketAddr, sync::Arc};
use tarpc::client;
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;
use tracing_futures::Instrument;

pub struct TradeClient {
    endpoint: quinn::Endpoint,
    remote_addr: SocketAddr,
    server_name: String,
    closed: bool,
    connection: Option<Arc<TradeConnection>>,
    rustls_config: rustls::ClientConfig,
}

impl TradeClient {
    pub async fn new(
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        server_name: &str,
        rustls_config: rustls::ClientConfig,
    ) -> Result<TradeClient, Box<dyn std::error::Error>> {
        let endpoint = quinn::Endpoint::client(local_addr)?;
        Ok(TradeClient {
            endpoint,
            remote_addr,
            server_name: server_name.to_owned(),
            closed: false,
            connection: None,
            rustls_config,
        })
    }

    pub async fn connect(&mut self) -> Result<Arc<TradeConnection>, Box<dyn std::error::Error>> {
        let client_config = ClientConfig::new(Arc::new(self.rustls_config.clone()));
        if self.connection.is_none() {
            info!("Establishing new connection");
            let new_connection = self
                .endpoint
                .connect_with(client_config, self.remote_addr, &self.server_name)?
                .instrument(tracing::info_span!("Establishing connection"))
                .await?;

            let quinn::NewConnection {
                connection: conn, ..
            } = new_connection;

            info!("Establishing bi-directional connection stream");
            let (send, recv) = conn
                .open_bi()
                .instrument(tracing::info_span!(
                    "Establishing bi-directional connection stream"
                ))
                .await?;

            let codec = LengthDelimitedCodec::new();
            let framed = Framed::new(StreamFramer { write: send, recv }, codec);
            let transport = tarpc::serde_transport::new(framed, Bincode::default());
            let client = FinancialServiceClient::new(client::Config::default(), transport).spawn();

            let connection = TradeConnection {
                conn,
                client,
                //current_request_id: Arc::new(AtomicU64::new(0)),
            };

            info!("Established bi-directional connection stream");

            let connection_arc = Arc::new(connection);
            //let connection_arc_clone = Arc::clone(&connection_arc);
            /*let fut = connection_arc_clone.serve_requests();
            info!("NOTEST");*/
            /*tokio::spawn(async move {
                info!("TEST TEST");
                match fut.await {
                    Ok(_) => (),
                    Err(e) => error!("Error in connection: {}", e),
                }
            });*/
            self.connection = Some(connection_arc);
        }

        if let Some(ref conn) = self.connection {
            info!("Using old connection");
            Ok(Arc::clone(&conn))
        } else {
            unreachable!()
        }
    }

    pub async fn close(&mut self) {
        self.endpoint
            .close(0u32.into(), b"Graceful connection disposal");

        // Give the server a fair chance to receive the close packet
        self.endpoint.wait_idle().await;
        self.closed = true;
    }
}

impl Drop for TradeClient {
    fn drop(&mut self) {
        if !self.closed {
            self.endpoint
                .close(0u32.into(), b"Hard connection disposal");
        }
    }
}

pub struct TradeConnection {
    conn: quinn::Connection,
    pub client: FinancialServiceClient,
    //bus: Mutex<Bus<EncodedPacket>>,
    //current_request_id: Arc<AtomicU64>,
}

impl TradeConnection {}

impl Drop for TradeConnection {
    fn drop(&mut self) {
        self.conn.close(
            0u32.into(),
            b"Connection disposal to reduce garbage coNNections",
        );
    }
}
