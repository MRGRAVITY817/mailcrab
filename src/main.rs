use {
    mailcrab::{configuration::get_configuration, startup::run},
    sqlx::PgPool,
    std::net::TcpListener,
    tracing::subscriber::set_global_default,
    tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer},
    tracing_log::LogTracer,
    tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Init logger
    LogTracer::init().expect("Failed to set logger");
    // This will filter the RUST_LOG env variable. Defaulted as `RUST_LOG=info`.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // This will format log from JSON to Bunyan. https://github.com/trentm/node-bunyan
    let formatting_layer = BunyanFormattingLayer::new("mailcrab".into(), std::io::stdout);
    // Subscriber will trace the spans, with awesome JSON formatted logs
    // which can be ingested easily by Elastic Search.
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");

    // Configuration
    let app_config = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&app_config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", app_config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
