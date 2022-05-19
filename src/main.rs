use {
    mailcrab::{
        configuration::get_config,
        email_client::EmailClient,
        startup::run,
        telemetry::{get_subscriber, init_subscriber},
    },
    sqlx::postgres::PgPoolOptions,
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
        .connect_lazy_with(app_config.database.with_db());

    let sender_email = app_config
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        app_config.email_client.base_url,
        sender_email,
        app_config.email_client.auth_token,
    );

    let address = format!(
        "{}:{}",
        app_config.application.host, app_config.application.port
    );

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool, email_client)?.await
}
