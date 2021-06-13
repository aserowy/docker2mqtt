use tokio::sync::{mpsc, oneshot};

use crate::configuration::Configuration;
use self::repository::LoggingRepository;
use tracing::error;

mod repository;

#[derive(Debug)]
pub struct UnixTimestamp {
    pub time: i64,
}

pub enum LoggingDbMessage {
    GetLastLoggingTime {
        respond_to: oneshot::Sender<Option<UnixTimestamp>>
    },
    SetLastLoggingTime {
        time: UnixTimestamp
    }
}

#[derive(Clone)]
pub struct LoggingDbHandle {
    sender: mpsc::Sender<LoggingDbMessage>
}

impl LoggingDbHandle {
    pub fn new(conf: &Configuration) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = LoggingDbActor::new(conf, receiver);
        tokio::spawn(actor.run());
        Self { sender }
    }

    pub async fn handle(&self, message: LoggingDbMessage) {
        self.sender
            .send(message)
            .await
            .map_err(|err| error!("Error sending LoggingRepositoryMessage: {}", err))
            .ok();
    }
}

struct LoggingDbActor {
    repository: Box<dyn LoggingRepository>,
    receiver: mpsc::Receiver<LoggingDbMessage>
}

impl LoggingDbActor {
    fn new(conf: &Configuration, receiver: mpsc::Receiver<LoggingDbMessage>) -> Self {
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

    fn handle(&mut self, message: LoggingDbMessage) {
        match message {
            LoggingDbMessage::GetLastLoggingTime { respond_to } => {
                if let Err(err) = respond_to.send(self.repository.get_last_logging_time()) {
                    error!("Error sending docker container list: {:?}", err)
                }
            }
            LoggingDbMessage::SetLastLoggingTime { time } => self.repository.set_last_logging_time(time)
        }
    }
}