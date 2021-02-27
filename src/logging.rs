use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::Layer, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry,
};

pub fn init() -> Vec<WorkerGuard> {
    LogTracer::init().expect("Unable to setup log tracer!");

    let mut guards = vec![];

    let (stdout_writer, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
    let stdout_logging_layer = Layer::new().with_writer(stdout_writer);
    guards.push(stdout_guard);

    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, "/docker2mqtt/logs", "d2m");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);
    let file_logging_layer = BunyanFormattingLayer::new(app_name, file_writer);
    guards.push(file_guard);

    let subscriber = Registry::default()
        .with(EnvFilter::new("INFO"))
        .with(stdout_logging_layer)
        .with(JsonStorageLayer)
        .with(file_logging_layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    guards
}
