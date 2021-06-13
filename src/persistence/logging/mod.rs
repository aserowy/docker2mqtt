use tokio::sync::{mpsc, oneshot};

use crate::configuration::Configuration;
use self::repository::LoggingRepository;

mod repository;

pub struct UnixTimestamp {
    pub time: i64,
}

pub enum LoggingRepositoryMessage {
    GetLastLoggingTime {
        respond_to: oneshot::Sender<UnixTimestamp>
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

    }
}