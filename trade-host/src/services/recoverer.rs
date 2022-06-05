use std::{io::SeekFrom, sync::Arc, time::Duration};

use crate::host::Host;

use super::Service;
use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};
use tracing::warn;
pub struct RecovererService {
    host: Arc<Host>,
    host_status: Arc<Mutex<Option<HostStatus>>>,
}

#[async_trait]
impl Service for RecovererService {
    type Params = ();
    async fn try_init(
        host: Arc<Host>,
        _params: Self::Params,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(RecovererService {
            host,
            host_status: Arc::new(Mutex::const_new(None)),
        }))
    }
    async fn run(
        self: Arc<Self>,
        _shutdown_recv: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let this = Arc::clone(&self);
        let host_status_file_path = self.host.config.misc_path.clone().join("~deeptrading.lock");
        let host_status_file = tokio::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(false)
            .open(host_status_file_path)
            .await
            .expect("Failed to open or create ~deeptrading.lock");
        let mut host_status = HostStatus::new(host_status_file);

        if host_status.get_value().await == true {
            warn!("The host was not terminated gracefully before.")
        }

        let host_status_arc = Arc::clone(&this.host_status);
        let mut host_status_lock = host_status_arc.lock().await;
        let _ = std::mem::replace(&mut *host_status_lock, Some(host_status));

        loop {
            tokio::task::yield_now().await;
            let mut host_status_lock = host_status_arc.lock().await;
            if let Some(host_status) = host_status_lock.as_mut() {
                host_status.set_value(true).await;
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}

pub struct HostStatus {
    pub file: tokio::fs::File,
}

impl HostStatus {
    pub fn new(file: tokio::fs::File) -> HostStatus {
        HostStatus { file }
    }
    pub async fn get_value(&mut self) -> bool {
        if self
            .file
            .metadata()
            .await
            .expect("Failed to get metadata")
            .len()
            == 0
        {
            return false;
        }
        self.file
            .seek(SeekFrom::Start(0))
            .await
            .expect("Failed to seek");
        self.file
            .read_u8()
            .await
            .expect("Failed to write status file")
            == 1
    }
    pub async fn set_value(&mut self, value: bool) {
        self.file
            .seek(SeekFrom::Start(0))
            .await
            .expect("Failed to seek");
        self.file
            .write_u8(if value { 1 } else { 0 })
            .await
            .expect("Failed to write status file");
        self.file.flush().await.expect("Failed to flush");
    }
}
