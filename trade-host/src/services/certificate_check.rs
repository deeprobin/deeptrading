use crate::{config::HostConfig, host::Host};
use async_trait::async_trait;
use rustls::{Certificate, PrivateKey};
use tokio::sync::broadcast::Receiver;

use std::{path::PathBuf, sync::Arc, time::Duration};
use tracing::{info, warn};
use x509_parser::{prelude::X509Certificate, traits::FromDer};

use super::Service;

pub struct CertificateCheckService {
    config: HostConfig,
}

impl CertificateCheckService {
    pub async fn get_certs(self: Arc<Self>) -> (Vec<Certificate>, PrivateKey) {
        let certificate_der_path = self.config.cert_path.clone().join("certificate.der");
        let certificate_der_data = tokio::fs::read(certificate_der_path)
            .await
            .expect("Failed to read certificate.der");
        let certificate_der = Certificate(certificate_der_data);

        let private_key_der_path = self.config.cert_path.clone().join("private_key.der");
        let private_key_der_data = tokio::fs::read(private_key_der_path)
            .await
            .expect("Failed to read private_key.der");

        let key = PrivateKey(private_key_der_data);
        return (vec![certificate_der], key);
    }

    // returns recommended duration for next check
    pub async fn check_and_regenerate(self: Arc<Self>) -> Duration {
        let certificate_pem_path = self.config.cert_path.clone().join("certificate.pem");
        let certificate_der_path = self.config.cert_path.clone().join("certificate.der");
        let private_key_pem_path = self.config.cert_path.clone().join("private_key.pem");
        let private_key_der_path = self.config.cert_path.clone().join("private_key.der");
        let request_pem_path = self.config.cert_path.clone().join("request.pem");
        let request_der_path = self.config.cert_path.clone().join("request.der");

        if !(certificate_pem_path.exists()
            && certificate_der_path.exists()
            && private_key_pem_path.exists()
            && private_key_der_path.exists()
            && request_pem_path.exists()
            && request_der_path.exists())
        {
            info!("No certificates found, generating new ones");
            self.generate(
                request_pem_path,
                request_der_path,
                private_key_pem_path,
                private_key_der_path,
                certificate_pem_path,
                certificate_der_path,
            )
            .await;

            // Check certificate after generation
            return Duration::ZERO;
        } else {
            info!("Certificates found, checking for validity");
            let certificate_data = tokio::fs::read(certificate_der_path.clone())
                .await
                .expect("Failed to read certificate.der");
            let certificate = X509Certificate::from_der(&certificate_data)
                .expect("Failed to read X509 certificate")
                .1;
            if certificate.validity.is_valid() {
                info!(
                    "Current certificate is valid until {until}",
                    until = certificate.validity.not_after.to_rfc2822()
                );
                let duration_until_expiration = certificate.validity.time_to_expiration().unwrap();
                let mut duration_until_regeneration = duration_until_expiration;

                // Regenerate 3 days before expiration
                duration_until_regeneration = duration_until_regeneration
                    .checked_sub(time::Duration::seconds(60 * 60 * 24 * 3))
                    .unwrap_or(time::Duration::ZERO);
                let sys_duration = std::time::Duration::from_millis(
                    duration_until_regeneration.whole_milliseconds() as u64,
                );
                return sys_duration;
            } else {
                warn!("Current certificates are not valid, generating new ones");
                self.generate(
                    request_pem_path,
                    request_der_path,
                    private_key_pem_path,
                    private_key_der_path,
                    certificate_pem_path,
                    certificate_der_path,
                )
                .await;
                // Check certificate after generation
                return Duration::ZERO;
            }
        }
    }
    pub async fn generate(
        self: Arc<Self>,
        request_pem: PathBuf,
        request_der: PathBuf,
        private_key_pem: PathBuf,
        private_key_der: PathBuf,
        certificate_pem: PathBuf,
        certificate_der: PathBuf,
    ) {
        let mut params = rcgen::CertificateParams::new(self.config.cert_names.clone());
        params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365 * 10);
        let certificate = rcgen::generate_simple_self_signed(self.config.cert_names.clone())
            .expect("Failed to generate self-signed certificate");

        tokio::fs::write(
            request_pem,
            certificate
                .serialize_pem()
                .expect("Failed to create request.pem"),
        )
        .await
        .expect("Failed to write request.pem");

        tokio::fs::write(
            request_der,
            certificate
                .serialize_der()
                .expect("Failed to create request.der"),
        )
        .await
        .expect("Failed to write request.der");

        tokio::fs::write(
            certificate_pem,
            certificate
                .serialize_pem()
                .expect("Failed to create certificate.pem"),
        )
        .await
        .expect("Failed to write certificate.pem");

        tokio::fs::write(
            certificate_der,
            certificate
                .serialize_der()
                .expect("Failed to create certificate.der"),
        )
        .await
        .expect("Failed to write certificate.der");

        tokio::fs::write(private_key_pem, certificate.serialize_private_key_pem())
            .await
            .expect("Failed to write private_key.pem");

        tokio::fs::write(private_key_der, certificate.serialize_private_key_der())
            .await
            .expect("Failed to write private_key.der");
    }
}

#[async_trait]
impl Service for CertificateCheckService {
    type Params = ();
    async fn try_init(
        host: Arc<Host>,
        _params: Self::Params,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        let arc = Arc::new(CertificateCheckService {
            config: host.config.clone(),
        });

        info!("Checking certificates");
        Arc::clone(&arc).check_and_regenerate().await;

        Ok(arc)
    }
    async fn run(
        self: Arc<Self>,
        mut shutdown_recv: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let this = Arc::clone(&self);

            // Check certificates, each minute
            let result = this.check_and_regenerate().await;
            if shutdown_recv.try_recv().is_ok() {
                return Ok(());
            }

            std::thread::sleep(result);
            tokio::task::yield_now().await;
        }
    }
}
