use chrono::Utc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::domain::{OperationalMode, OperatorIntent};

#[derive(Clone)]
pub struct CommandBus {
    sender: mpsc::Sender<OperatorIntent>,
}

impl CommandBus {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<OperatorIntent>) {
        let (sender, receiver) = mpsc::channel(buffer);
        (Self { sender }, receiver)
    }

    pub async fn publish(
        &self,
        actor: impl Into<String>,
        mode: OperationalMode,
        market: impl Into<String>,
        content: impl Into<String>,
        requested_capabilities: Vec<String>,
    ) -> Result<Uuid, mpsc::error::SendError<OperatorIntent>> {
        let intent = OperatorIntent {
            id: Uuid::new_v4(),
            actor: actor.into(),
            mode,
            market: market.into(),
            content: content.into(),
            requested_capabilities,
            created_at: Utc::now(),
        };
        let id = intent.id;
        self.sender.send(intent).await?;
        Ok(id)
    }
}
