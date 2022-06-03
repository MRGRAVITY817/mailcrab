use {
    crate::{
        authentication::reject_anonymous_users,
        configuration::{DatabaseSettings, Settings},
        email_client::EmailClient,
        routes::{
            admin_dashboard, change_password, change_password_form, confirm, health_check, home,
            log_out, login_form, login_submit, publish_issue, publish_issue_form,
            publish_newsletter, subscribe,
        },
    },
    actix_session::{storage::RedisSessionStore, SessionMiddleware},
    actix_web::{cookie::Key, dev::Server, web, App, HttpServer},
    actix_web_flash_messages::{storage::CookieMessageStore, FlashMessagesFramework},
    actix_web_lab::middleware::from_fn,
    secrecy::{ExposeSecret, Secret},
    sqlx::{postgres::PgPoolOptions, PgPool},
    std::net::TcpListener,
    tracing_actix_web::TracingLogger,
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(app_config: Settings) -> Result<Self, anyhow::Error> {
        let db_pool = get_db_pool(&app_config.database);
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
        let base_url = app_config.application.base_url;
        let port = listener.local_addr().unwrap().port();
        let hmac_secret = app_config.application.hmac_secret;
        let redis_uri = app_config.redis_uri;
        let server = run(
            listener,
            db_pool,
            email_client,
            base_url,
            hmac_secret,
            redis_uri,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

/// Run http server with user settings
async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    // `web::Data` is basically an `Arc`, which will safely share the app state across threads
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    // Storing secured flash message
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    // This will go into middleware
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login_submit))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_anonymous_users))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/newsletter", web::get().to(publish_issue_form))
                    .route("/newsletter", web::post().to(publish_issue))
                    .route("/password", web::get().to(change_password_form))
                    .route("/password", web::post().to(change_password))
                    .route("/logout", web::post().to(log_out)),
            )
            .route("/newsletters", web::post().to(publish_newsletter))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(web::Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

/// A wrapper type to avoid conflict with other `web::Data<Secret<String>>` states.
#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

pub fn get_db_pool(db_config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(db_config.with_db())
}
