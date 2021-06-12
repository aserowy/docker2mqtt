use std::sync::Arc;

use tokio::{
    sync::{mpsc, Mutex},
    task,
};
use tracing::error;

#[derive(Debug)]
pub struct Multiplier<T: Send + 'static> {
    pub receiver: mpsc::Receiver<T>,
    sender_arc: Arc<Mutex<Vec<mpsc::Sender<T>>>>,
}

impl<T: Clone + Send + 'static> Multiplier<T> {
    pub async fn new(mut receiver: mpsc::Receiver<T>) -> Self {
        let (sender, multiplier_receiver) = mpsc::channel(50);
        let sender_arc = Arc::new(Mutex::new(vec![sender]));

        let sender_arc_clone = sender_arc.clone();
        task::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let sender_locked = sender_arc_clone.lock().await;
                // TODO: rework to parallel sending
                for sndr in sender_locked.iter() {
                    let msg = message.clone();
                    if let Err(e) = sndr.send(msg).await {
                        error!("message was not sent: {}", e);
                    }
                }
            }
        });

        Multiplier {
            sender_arc,
            receiver: multiplier_receiver,
        }
    }

    pub async fn clone(&self) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let mut sender_locked = self.sender_arc.lock().await;
        sender_locked.push(sender);

        Multiplier {
            sender_arc: self.sender_arc.clone(),
            receiver,
        }
    }
}
