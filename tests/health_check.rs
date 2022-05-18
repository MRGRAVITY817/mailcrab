use {mailcrab::configuration::get_configuration, sqlx::PgPool, std::net::TcpListener};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let app_config = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPool::connect(&app_config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let server = mailcrab::startup::run(listener, db_pool.clone()).expect("Failed to bind address");
    // Lauch the server as a background task
    let _ = tokio::spawn(server);
    TestApp { address, db_pool }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=hoon%20wee&email=mrgravity817%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-from-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");
    assert_eq!(saved.email, "mrgravity817@gmail.com");
    assert_eq!(saved.name, "hoon");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=hoon%20wee", "missing the email"),
        ("email=mrgravity817%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-from-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
