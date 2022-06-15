use mailcrab::issue_delivery_worker::{try_execute_task, ExecutionOutcome};

use {
    argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version},
    mailcrab::{
        configuration::{get_config, DatabaseSettings},
        email_client::EmailClient,
        startup::{get_db_pool, Application},
        telemetry::{get_subscriber, init_subscriber},
    },
    once_cell::sync::Lazy,
    sqlx::{Connection, Executor, PgConnection, PgPool},
    uuid::Uuid,
    wiremock::MockServer,
};

// Subscriber should be created once ()
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

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub port: u16,
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
    pub email_client: EmailClient,
}

impl TestApp {
    /// Consume all the messages in queue
    pub async fn dispatch_all_pending_emails(&self) {
        loop {
            if let ExecutionOutcome::EmptyQueue =
                try_execute_task(&self.db_pool, &self.email_client)
                    .await
                    .unwrap()
            {
                break;
            }
        }
    }

    /// Return given route prepended with test app's address
    fn app_route(&self, route: &str) -> String {
        format!("{}/{}", self.address, route)
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(self.app_route("subscriptions"))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| -> reqwest::Url {
            let links = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| l.kind() == &linkify::LinkKind::Url)
                .collect::<Vec<_>>();

            // Check if there's a single link in body
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            // Check if host is localhost
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    /// Post newsletters to subscribed user
    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        self.api_client
            .post(self.app_route("newsletters"))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Post login request
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        // For returning redirection, we use `Client::builder().redirect()`
        self.api_client
            .post(self.app_route("login"))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Get HTML text from login page
    pub async fn get_login_html(&self) -> String {
        self.api_client
            .get(self.app_route("login"))
            .send()
            .await
            .expect("Failed to execute query")
            .text()
            .await
            .unwrap()
    }

    /// Get response from `admin/dashboard`
    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.api_client
            .get(self.app_route("admin/dashboard"))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Get HTML string from `admin/dashboard`
    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard().await.text().await.unwrap()
    }

    /// Get response from `admin/password`
    pub async fn get_change_password(&self) -> reqwest::Response {
        self.api_client
            .get(self.app_route("admin/password"))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Get HTML string from response of `admin/password`
    pub async fn get_change_password_html(&self) -> String {
        self.get_change_password().await.text().await.unwrap()
    }

    /// Post request for changing admin password from `admin/password`
    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(self.app_route("admin/password"))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Post request for logging user out
    pub async fn post_logout(&self) -> reqwest::Response {
        self.api_client
            .post(self.app_route("admin/logout"))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Get admin newsletter form from `admin/newsletter`
    pub async fn get_admin_newsletter(&self) -> reqwest::Response {
        self.api_client
            .get(self.app_route("admin/newsletter"))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// GET HTML string from `admin/newsletter` page
    pub async fn get_admin_newsletter_html(&self) -> String {
        self.get_admin_newsletter().await.text().await.unwrap()
    }

    /// Publish issue with title, text/html content
    pub async fn post_publish_issue<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(self.app_route("admin/newsletter"))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    pub async fn login(&self, app: &TestApp) {
        app.post_login(&serde_json::json!({
            "username": &self.username,
            "password": &self.password,
        }))
        .await;
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // Match parameters of the default password
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash) VALUES ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash
        )
        .execute(pool)
        .await
        .expect("Failed to store test user");
    }
}

pub async fn spawn_app() -> TestApp {
    // Init tracing subscriber
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let app_config = {
        let mut c = get_config().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&app_config.database).await;

    let application = Application::build(app_config.clone())
        .await
        .expect("Failed to build application");
    let port = application.port();
    let address = format!("http://127.0.0.1:{}", port);
    let db_pool = get_db_pool(&app_config.database);
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        port,
        address,
        db_pool,
        email_server,
        test_user: TestUser::generate(),
        api_client: client,
        email_client: app_config.email_client.client(),
    };

    test_app.test_user.store(&test_app.db_pool).await;

    test_app
}

/// Configure Postgres database
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

/// Commonly used assert test for redirection
pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
