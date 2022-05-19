use sqlx::postgres::PgPoolOptions;

use {
    mailcrab::{
        configuration::get_config,
        startup::run,
        telemetry::{get_subscriber, init_subscriber},
    },
    secrecy::ExposeSecret,
    std::net::TcpListener,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Init subscriber
    let subscriber = get_subscriber("mailcrab".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Configuration
    let app_config = get_config().expect("Failed to read configuration");

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&app_config.database.connection_string().expose_secret())
        .expect("Failed to create Postgres connection pool.");

    let address = format!(
        "{}:{}",
        app_config.application.host, app_config.application.port
    );

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
