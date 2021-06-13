use tokio::sync::{mpsc, oneshot};

use crate::configuration::Configuration;
use self::repository::LoggingRepository;
use tracing::error;

mod repository;

#[derive(Debug)]
pub struct UnixTimestamp {
    pub time: i64,
}

pub enum LoggingRepositoryMessage {
    GetLastLoggingTime {
        respond_to: oneshot::Sender<Option<UnixTimestamp>>
    },
    SetLastLoggingTime {
        time: UnixTimestamp
    }
}

#[derive(Clone)]
pub struct LoggingRepositoryHandle {
    sender: mpsc::Sender<LoggingRepositoryMessage>
}

impl LoggingRepositoryHandle {
    pub fn new(conf: &Configuration) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = LoggingRepositoryActor::new(conf, receiver);
        tokio::spawn(actor.run());
        Self { sender }
    }

    pub async fn handle(&self, message: LoggingRepositoryMessage) {
        self.sender
            .send(message)
            .await
            .map_err(|err| error!("Error sending LoggingRepositoryMessage: {}", err))
            .ok();
    }
}

struct LoggingRepositoryActor {
    repository: Box<dyn LoggingRepository>,
    receiver: mpsc::Receiver<LoggingRepositoryMessage>
}

impl LoggingRepositoryActor {
    fn new(conf: &Configuration, receiver: mpsc::Receiver<LoggingRepositoryMessage>) -> Self {
        Self {
            repository: repository::new(conf),
            receiver
        }
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message);
        }
    }

    fn handle(&mut self, message: LoggingRepositoryMessage) {
        match message {
            LoggingRepositoryMessage::GetLastLoggingTime { respond_to } => {
                if let Err(err) = respond_to.send(self.repository.get_last_logging_time()) {
                    error!("Error sending docker container list: {:?}", err)
                }
            }
            LoggingRepositoryMessage::SetLastLoggingTime { time } => self.repository.set_last_logging_time(time)
        }
    }
}