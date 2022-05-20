use {
    mailcrab::{
        configuration::{get_config, DatabaseSettings},
        email_client::EmailClient,
        telemetry::{get_subscriber, init_subscriber},
    },
    once_cell::sync::Lazy,
    sqlx::{Connection, Executor, PgConnection, PgPool},
    std::net::TcpListener,
    uuid::Uuid,
};

// Subscriber should be created once (singleton pattern)
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    // Init subscriber by forcing our lazy TRACING
    Lazy::force(&TRACING);

    // TcpListener setting
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Database pool setting
    let mut app_config = get_config().expect("Failed to read configuration.");
    app_config.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&app_config.database).await;

    // Email client setting
    let sender_email = app_config
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = app_config.email_client.timeout();
    let email_client = EmailClient::new(
        app_config.email_client.base_url,
        sender_email,
        app_config.email_client.auth_token,
        timeout,
    );

    // Launch the server as a background task
    let server = mailcrab::startup::run(listener, db_pool.clone(), email_client)
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { address, db_pool }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database");

    db_pool
}
