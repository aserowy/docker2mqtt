use tokio::sync::mpsc;
use tracing::error;

#[derive(Debug)]
pub struct Reducer<T: Send + 'static> {
    pub receiver: mpsc::Receiver<T>,
}

impl<T: Send + 'static> Reducer<T> {
    pub async fn new(receivers: Vec<mpsc::Receiver<T>>) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        for mut rcvr in receivers.into_iter() {
            let sndr = sender.clone();

            tokio::spawn(async move {
                while let Some(message) = rcvr.recv().await {
                    if let Err(e) = sndr.send(message).await {
                        error!("message was not sent: {}", e);
                    }
                }
            });
        }

        Reducer { receiver }
    }
}
