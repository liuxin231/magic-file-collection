use crate::app_config::Settings;
use crate::watching::async_watch;
use crate::watching::offset::flush;
use std::io;
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

mod app_config;
mod watching;

#[tokio::main]
async fn main() {
    let settings: Arc<Settings> = Arc::new(
        config::Config::builder()
            .add_source(config::File::with_name("./config/settings.toml").required(false))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap(),
    );
    let file_appender =
        tracing_appender::rolling::daily(&settings.log.directory, &settings.log.file_name_prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with(fmt::Layer::new().with_writer(io::stdout))
        .with(fmt::Layer::new().with_writer(non_blocking));
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");

    let path = &settings.notify.watching_path;
    if path.is_empty() {
        tracing::error!("watching path list is empty.");
        return;
    }
    tracing::info!("watching path list: {:?}", &path);
    tokio::spawn(flush(settings.clone()));
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let path = path.to_string();
    tokio::spawn(async move {
        if let Err(error) = async_watch(path, tx).await {
            tracing::error!("watching path error: {}", error.to_string());
        }
    });
    while let Some(message) = rx.recv().await {
        tracing::info!("{}", message);
    }
}
