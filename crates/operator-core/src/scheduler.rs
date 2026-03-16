use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info};

#[derive(Clone, Default)]
pub struct BackgroundScheduler {
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl BackgroundScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn spawn_job<F, Fut>(&self, name: impl Into<String>, interval: Duration, job: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let name = name.into();
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                match job().await {
                    Ok(()) => info!(job = %name, "scheduled job completed"),
                    Err(error_message) => {
                        error!(job = %name, error = %error_message, "scheduled job failed")
                    }
                }
            }
        });
        self.handles.lock().await.push(handle);
    }

    pub async fn shutdown(&self) {
        let mut handles = self.handles.lock().await;
        for handle in handles.drain(..) {
            handle.abort();
        }
    }
}
