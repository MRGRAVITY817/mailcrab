use {
    crate::{
        configuration::{DatabaseSettings, Settings},
        email_client::EmailClient,
        routes::{health_check, subscribe},
    },
    actix_web::{dev::Server, web, App, HttpServer},
    sqlx::{postgres::PgPoolOptions, PgPool},
    std::net::TcpListener,
    tracing_actix_web::TracingLogger,
};

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // `web::Data` is basically `Arc`, which will safely share the app state across threads
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub async fn build(app_config: Settings) -> Result<Server, std::io::Error> {
    // Postgres connection pool
    let db_pool = get_db_pool(&app_config.database);

    // Email client
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

    let address = format!(
        "{}:{}",
        app_config.application.host, app_config.application.port
    );

    let listener = TcpListener::bind(address)?;
    run(listener, db_pool, email_client)
}

pub fn get_db_pool(db_config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(db_config.with_db())
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(app_config: Settings) -> Result<Self, std::io::Error> {
        let db_pool = get_db_pool(&app_config.database);
        // Email client
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

        let address = format!(
            "{}:{}",
            app_config.application.host, app_config.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
