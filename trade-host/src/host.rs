use crate::config::HostConfig;
use crate::services::ServiceStatus;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::broadcast::{self, Receiver, Sender};

use tracing::{error, info, info_span, trace, warn};
use tracing_futures::Instrument;

use crate::services::{
    binance::BinanceService, certificate_check::CertificateCheckService, k8s::KubernetesService,
    recoverer::RecovererService, start_service, trade_protocol::TradeProtocolService, Service,
};

pub struct Host {
    pub config: HostConfig,
}

impl Host {
    pub fn new(config: HostConfig) -> Arc<Host> {
        Arc::new(Host {
            config: config.clone(),
        })
    }

    pub async fn run(
        self: Arc<Self>,
        shutdown_sender: Sender<()>,
        mut shutdown_recv: Receiver<()>,
    ) {
        ensure_directory(&self.config.cert_path).await;
        ensure_directory(&self.config.misc_path).await;

        let this = Arc::clone(&self);

        let cert_service = try_init::<CertificateCheckService>(Arc::clone(&this), ()).await;
        let binance_service = try_init::<BinanceService>(Arc::clone(&this), (None, None)).await;
        let protocol_service = try_init::<TradeProtocolService>(
            Arc::clone(&this),
            (Arc::clone(&binance_service), Arc::clone(&cert_service)),
        )
        .await;

        let recoverer_service = try_init::<RecovererService>(Arc::clone(&this), ()).await;
        let k8s_service = try_init::<KubernetesService>(Arc::clone(&this), ()).await;

        let mut services = Vec::with_capacity(5);

        services.push(start_service_guarded(protocol_service, &shutdown_sender));
        services.push(start_service_guarded(recoverer_service, &shutdown_sender));
        services.push(start_service_guarded(binance_service, &shutdown_sender));
        services.push(start_service_guarded(cert_service, &shutdown_sender));
        services.push(start_service_guarded(k8s_service, &shutdown_sender));

        shutdown_recv
            .recv()
            .await
            // Ignore failure, we're shutting down anyway
            .ok();

        info!("Shutting down");

        let shutdown_successful = Arc::new(AtomicBool::new(true));
        for service in services {
            let shutdown_successful_arc = Arc::clone(&shutdown_successful);
            let service_name = service.service_name.clone();
            let wait_task = async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                warn!(
                    "Shutdown of service {service} takes longer as expected",
                    service = service_name
                );

                tokio::time::sleep(Duration::from_secs(2)).await;
                shutdown_successful_arc.store(false, Ordering::Relaxed);
                error!(
                    "Waiting for shutdown of service {service} canceled",
                    service = service_name
                );
            };
            tokio::select! {
                _ = service.wait_for_service_handle() => {},
                _ = wait_task => {}
            }
        }
        std::process::exit(if shutdown_successful.load(Ordering::Relaxed) {
            0
        } else {
            -1
        });
    }
}

async fn ensure_directory<P: AsRef<Path> + Clone>(directory: P) {
    if !tokio::fs::metadata(directory.clone()).await.is_ok() {
        match tokio::fs::create_dir_all(&directory).await {
            Ok(_) => info!("Created directory: {:?}", directory.as_ref().display()),
            Err(e) => {
                error!("Failed to create directory: {:?}", e);
                std::process::exit(-1);
            }
        }
    }
}

async fn try_init<T: Service>(host: Arc<Host>, params: T::Params) -> Arc<T> {
    let service_name = std::any::type_name::<T>();
    let span = info_span!("Service Initialization", service_name = service_name);
    async {
        let service_result = T::try_init(host, params).await;
        match service_result {
            Ok(service) => {
                info!("Initialized service: {:?}", service_name);
                service
            }
            Err(err) => {
                error!("Failed to initialize service: {:?}", err);
                std::process::exit(-1)
            }
        }
    }
    .instrument(span)
    .await
}

struct ServiceHandle {
    status: Arc<AtomicU8>,
    service_name: String,
}

impl ServiceHandle {
    async fn wait_for_service_handle(&self) {
        loop {
            let status: ServiceStatus =
                unsafe { std::mem::transmute(self.status.load(Ordering::Relaxed)) };

            if status == ServiceStatus::Finished || status == ServiceStatus::Failed {
                break;
            }

            tokio::task::yield_now().await;
        }
    }
}

fn start_service_guarded<S: 'static + Service>(
    service: Arc<S>,
    shutdown_sender: &Sender<()>,
) -> ServiceHandle {
    let service_name = std::any::type_name::<S>();

    let (status_sender, mut status_receiver) = broadcast::channel(24);
    let status_arc = Arc::new(AtomicU8::new(ServiceStatus::Started as u8));
    let cloned_status_arc = Arc::clone(&status_arc);

    tokio::spawn(async move {
        loop {
            match status_receiver.recv().await {
                Ok(service_status) => {
                    let status: ServiceStatus =
                        unsafe { std::mem::transmute(cloned_status_arc.load(Ordering::Relaxed)) };
                    cloned_status_arc.store(service_status as u8, Ordering::Relaxed);

                    trace!(
                        "Received service status: {:?} of {}",
                        service_status,
                        service_name
                    );

                    if status == ServiceStatus::Finished || status == ServiceStatus::Failed {
                        break;
                    }
                }
                Err(_) => {
                    cloned_status_arc.store(ServiceStatus::Finished as u8, Ordering::Relaxed);
                }
            }
        }
    });

    let cloned_shutdown_sender = shutdown_sender.clone();
    let cloned_status_sender = status_sender.clone();

    tokio::spawn(async move {
        start_service(service, &cloned_shutdown_sender, cloned_status_sender)
            .expect(&format!("Failed to start service {}", service_name));
    });

    ServiceHandle {
        status: status_arc,
        service_name: service_name.to_string(),
    }
}
