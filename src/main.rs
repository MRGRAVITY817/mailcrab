use {
    mailcrab::{
        configuration::get_configuration,
        startup::run,
        telemetry::{get_subscriber, init_subscriber},
    },
    sqlx::PgPool,
    std::net::TcpListener,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Init subscriber
    let subscriber = get_subscriber("mailcrab".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Configuration
    let app_config = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&app_config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", app_config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
