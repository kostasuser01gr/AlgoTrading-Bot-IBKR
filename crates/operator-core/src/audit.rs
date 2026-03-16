use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::domain::AuditEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEnvelope {
    pub event: AuditEvent,
    pub previous_hash: Option<String>,
    pub record_hash: String,
    pub written_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("failed to serialize audit event: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to persist audit event: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct TamperEvidentAuditWriter {
    path: PathBuf,
    previous_hash: Arc<Mutex<Option<String>>>,
}

impl TamperEvidentAuditWriter {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self { path: path.as_ref().to_path_buf(), previous_hash: Arc::new(Mutex::new(None)) }
    }

    pub async fn write(&self, event: AuditEvent) -> Result<AuditEnvelope, AuditError> {
        let event_json = serde_json::to_string(&event)?;
        let mut previous_hash = self.previous_hash.lock().await;
        let record_hash = hash_record(previous_hash.as_deref(), &event_json);
        let envelope = AuditEnvelope {
            event,
            previous_hash: previous_hash.clone(),
            record_hash: record_hash.clone(),
            written_at: Utc::now(),
        };
        let serialized = serde_json::to_string(&envelope)?;

        let mut file = OpenOptions::new().create(true).append(true).open(&self.path).await?;
        file.write_all(serialized.as_bytes()).await?;
        file.write_all(b"\n").await?;
        *previous_hash = Some(record_hash);
        Ok(envelope)
    }
}

fn hash_record(previous_hash: Option<&str>, event_json: &str) -> String {
    let mut hasher = Sha256::new();
    if let Some(previous_hash) = previous_hash {
        hasher.update(previous_hash.as_bytes());
    }
    hasher.update(event_json.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::{AuditEvent, OperationalMode};

    use super::TamperEvidentAuditWriter;

    #[tokio::test]
    async fn audit_writer_chains_hashes() {
        let path = std::env::temp_dir().join(format!("audit-{}.jsonl", Uuid::new_v4()));
        let writer = TamperEvidentAuditWriter::new(&path);

        let first = AuditEvent {
            id: Uuid::new_v4(),
            actor: "tester".to_string(),
            action: "bootstrap".to_string(),
            mode: OperationalMode::Research,
            status: "ok".to_string(),
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            details: serde_json::json!({"step": 1}),
        };
        let second = AuditEvent {
            id: Uuid::new_v4(),
            actor: "tester".to_string(),
            action: "bootstrap".to_string(),
            mode: OperationalMode::Research,
            status: "ok".to_string(),
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            details: serde_json::json!({"step": 2}),
        };

        let first_envelope = writer.write(first).await.expect("first write");
        let second_envelope = writer.write(second).await.expect("second write");

        assert_eq!(second_envelope.previous_hash, Some(first_envelope.record_hash));

        let content = fs::read_to_string(&path).expect("read audit log");
        assert_eq!(content.lines().count(), 2);
        let _ = fs::remove_file(path);
    }
}
