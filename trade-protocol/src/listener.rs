use crate::{
    //    encoding::{self, EncodedPacket},
    packets::{AllocationResponsePacketData, HeartbeatPacketData},
    services::{FinancialServer, FinancialService, FinancialServiceHandler},
    StreamFramer,
};
use async_trait::async_trait;
//use chrono::Utc;
use futures_util::stream::StreamExt;
use quinn::{Connection, ServerConfig};
//use quinn::{RecvStream, SendStream, ServerConfig};
//use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tarpc::server::{self, Channel};
use tokio::sync::Mutex;
use tokio_serde::formats::Bincode;
use tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio_util::codec::Framed;
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;

#[async_trait]
pub trait TradeListenerHandler: Send {
    async fn on_connect(
        &mut self,
        connection: Arc<quinn::Connection>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn handle_heartbeat(
        &mut self,
        connection: Arc<quinn::Connection>,
        packet: HeartbeatPacketData,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn handle_allocation_request(
        &mut self,
        connection: Arc<quinn::Connection>,
    ) -> Result<AllocationResponsePacketData, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct TradeListener {
    server_config: ServerConfig,
}

impl TradeListener {
    pub fn new(
        certs: Vec<rustls::Certificate>,
        key: rustls::PrivateKey,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
        //server_crypto.alpn_protocols = common::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

        let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));
        Arc::get_mut(&mut server_config.transport)
            .unwrap()
            .max_concurrent_uni_streams(0_u8.into());
        Ok(TradeListener { server_config })
    }

    pub async fn listen<
        'a,
        H: 'static + Send + FinancialServiceHandler + Sync,
        F: 'static + Send + Fn(Arc<Connection>) -> Arc<H> + std::marker::Sync,
    >(
        &mut self,
        addr: SocketAddr,
        create_handler: Arc<F>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (endpoint, mut incoming) = quinn::Endpoint::server(self.server_config.clone(), addr)?;
        info!("listening on {}", endpoint.local_addr()?);

        while let Some(conn) = incoming.next().await {
            info!("Incoming connection from {}", conn.remote_address());

            let fut = handle_connection(conn, Arc::clone(&create_handler));
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    error!("connection failed: {reason}", reason = e.to_string())
                }
            });
        }

        Ok(())
    }
}

async fn handle_connection<
    H: FinancialServiceHandler + Send + Sync + 'static,
    F: Send + Fn(Arc<Connection>) -> Arc<H>,
>(
    conn: quinn::Connecting,
    create_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error>> {
    let quinn::NewConnection {
        connection,
        mut bi_streams,
        ..
    } = conn.await?;
    let span = info_span!(
        "connection",
        remote = %connection.remote_address(),
        protocol = %connection
            .handshake_data()
            .unwrap()
            .downcast::<quinn::crypto::rustls::HandshakeData>().unwrap()
            .protocol
            .map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned())
    );
    let connection_arc = Arc::new(connection);
    //tarpc::serde_transport::new(, codec);
    async {
        info!("established");

        // Each stream initiated by the client constitutes a new request.
        while let Some(stream) = bi_streams.next().await {
            let stream = match stream {
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("connection closed");
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(s) => s,
            };

            let codec = LengthDelimitedCodec::new();
            let framed = Framed::new(
                StreamFramer {
                    write: stream.0,
                    recv: stream.1,
                },
                codec,
            );
            info!("established bi-stream");
            let transport = tarpc::serde_transport::new(framed, Bincode::default());
            info!("Initialized transport");
            let channel = server::BaseChannel::with_defaults(transport);
            info!("Initialized channel");
            let handler = create_handler(connection_arc.clone());
            info!("Initialized handler");
            let server = FinancialServer(connection_arc.clone(), handler);
            info!("Serving requests");
            tokio::spawn(async move {
                channel.execute(server.serve());
            });
            /*
            //tokio::spawn(fut);
            let fut = handle_request(Arc::clone(&connection_arc), stream, Arc::clone(&handler));
            tokio::spawn(
                async move {
                    if let Err(e) = fut.await {
                        error!("failed: {reason}", reason = e.to_string());
                    }
                }, // .instrument(info_span!("request")),
            );*/
        }
        Ok(())
    }
    .instrument(span)
    .await?;
    Ok(())
}

/*
async fn handle_request<T: TradeListenerHandler>(
    connection: Arc<quinn::Connection>,
    (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
    handler: Arc<Mutex<T>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Incoming request");
    let encoded_packet = EncodedPacket::read(&mut recv).await?;
    let packet = crate::encoding::Packet::decode(&encoded_packet)?;

    let mut handler = handler.lock().await;

    match packet {
        crate::encoding::Packet::Heartbeat(packet) => {
            handler.handle_heartbeat(connection, packet).await?;
            let response_packet = encoding::Packet::Heartbeat(HeartbeatPacketData {
                unix_ms: Utc::now().timestamp_millis(),
            })
            .encode(encoded_packet.identifier);

            response_packet.write(&mut send).await?;
            send.flush().await?;
        }
        crate::encoding::Packet::AllocationRequest(_) => {
            info!("Incoming allocation request");
            let result = handler.handle_allocation_request(connection).await?;
            let response_packet =
                encoding::Packet::AllocationResponse(result).encode(encoded_packet.identifier);
            response_packet.write(&mut send).await?;
            send.flush().await?;
        }
        packet => {
            error!("unhandled packet: {:?}", packet);
        }
    }

    Ok(())
}
*/
