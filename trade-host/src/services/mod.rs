use crate::host::Host;
use async_trait::async_trait;
use futures_util::future::{AbortHandle, Abortable};
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::{error, error_span, info, info_span, trace_span};
use tracing_futures::Instrument;

pub mod binance;
pub mod certificate_check;
pub mod k8s;
pub mod recoverer;
pub mod trade_protocol;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceStatus {
    Started,
    Finished,
    Failed,
}

#[async_trait]
pub trait Service: Send + Sync {
    type Params;
    async fn try_init(
        host: Arc<Host>,
        params: Self::Params,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>>;
    async fn run(
        self: Arc<Self>,
        mut shutdown_recv: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub(crate) fn start_service<S: 'static + Service>(
    service: Arc<S>,
    shutdown_sender: &Sender<()>,
    status_sender: Sender<ServiceStatus>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service_name = std::any::type_name::<S>();

    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let shutdown_recv = shutdown_sender.subscribe();
    let abortable_sender = status_sender.clone();
    let abortable = Abortable::new(
        async move {
            abortable_sender
                .send(ServiceStatus::Started)
                // do not panic if there are no active receivers
                .ok();

            match service
                .run(shutdown_recv)
                .instrument(info_span!("Service Lifetime", service_name = service_name))
                .await
            {
                Ok(()) => {
                    info_span!("Service Shutdown", service_name = service_name).in_scope(|| {
                        info!("Service {} finished", service_name);
                        abortable_sender
                            .send(ServiceStatus::Finished)
                            // do not panic if there are no active receivers
                            .ok();
                    });
                }
                Err(e) => {
                    error_span!("Service Shutdown", service_name = service_name).in_scope(|| {
                        error!("Service failed: {}", e);
                        abortable_sender
                            .send(ServiceStatus::Failed)
                            // do not panic if there are no active receivers
                            .ok();
                    });
                }
            };
        },
        abort_registration,
    );

    let mut cloned_receiver = shutdown_sender.subscribe();
    let cloned_status_sender = status_sender.clone();
    tokio::spawn(async move {
        let _ = cloned_receiver
            .recv()
            .await
            .expect("Failed to receive shutdown signal");
        abort_handle.abort();
        cloned_status_sender
            .send(ServiceStatus::Finished)
            // do not panic if there are no active receivers
            .ok();
    });

    tokio::spawn(async move {
        match abortable
            .instrument(trace_span!(
                "Service Abortion Listener",
                service_name = service_name
            ))
            .await
        {
            Ok(_) => {
                info!("Service {} finished", service_name);
                status_sender
                    .send(ServiceStatus::Finished)
                    // do not panic if there are no active receivers
                    .ok();
            }
            Err(_aborted) => {
                info!("Service {} aborted", service_name);
                status_sender
                    .send(ServiceStatus::Finished)
                    // do not panic if there are no active receivers
                    .ok();
            }
        };
    });
    Ok(())
}
