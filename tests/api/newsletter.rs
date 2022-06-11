use {
    crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp},
    std::time::Duration,
    uuid::Uuid,
    wiremock::{
        matchers::{any, method, path},
        Mock, ResponseTemplate,
    },
};

#[tokio::test]
async fn newsletter_returns_400_for_invalid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = test_app.post_newsletters(invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let test_app = spawn_app().await;
    create_unconfirmed_subscriber(&test_app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let response = test_app.post_newsletters(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let response = test_app.post_newsletters(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_subscriber(test_app: &TestApp) -> ConfirmationLinks {
    let body = "name=hoon%2wee&email=mrgravity817%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&test_app.email_server)
        .await;

    test_app
        .post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &test_app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    test_app.get_confirmation_links(email_request)
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let test_app = spawn_app().await;
    let response = test_app
        .api_client
        .post(&format!("{}/newsletters", &test_app.address))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

async fn create_confirmed_subscriber(test_app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(test_app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    // Arrange
    let test_app = spawn_app().await;

    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    // Act
    let response = test_app
        .api_client
        .post(&format!("{}/newsletters", &test_app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    // Arrange
    let test_app = spawn_app().await;
    let username = &test_app.test_user.username;
    let password = Uuid::new_v4().to_string();
    // Check if the test user's password is accidentally same with random password
    assert_ne!(test_app.test_user.password, password);

    // Act
    let response = test_app
        .api_client
        .post(&format!("{}/newsletters", &test_app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    // Create a mock email server that has `POST /email` API
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1) // works only once
        .mount(&test_app.email_server)
        .await;

    // Act & Assert
    // 1. Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter Title",
        "text_content": "Newsletter Text Content",
        "html_content": "<p>Newsletter HTML content</p>",
        // we expect the idempotency_key as part of form data, not as a header
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = test_app.post_publish_issue(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // 2. Follow the redirect
    let html_page = test_app.get_admin_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));

    // 3. Submit newsletter form **again**
    let response = test_app.post_publish_issue(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // 4. Follow the redirect
    let html_page = test_app.get_admin_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
}

#[tokio::test]
async fn should_be_logged_in_to_reach_admin_newsletter_page() {
    // Arrange
    let test_app = spawn_app().await;
    // Act
    let response = test_app.get_admin_newsletter().await;
    // Assert
    // Redirects to `login` when user is not logged in
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn should_be_logged_in_to_publish_newsletter_issue() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let response = test_app
        .post_publish_issue(&serde_json::json!({
            "title": "Title",
            "text_content": "Text Content",
            "html_content": "<p>Html Content</p>",
            "idempotency_key": uuid::Uuid::new_v4().to_string(),
        }))
        .await;

    // Assert
    // Redirects to `login` when user is not logged in
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn should_publish_newsletter_issue() {
    // Arrange
    let test_app = spawn_app().await;

    // Act & Assert
    // 1. Login
    let response = test_app
        .post_login(&serde_json::json!({
                "username": &test_app.test_user.username,
                "password": &test_app.test_user.password,
        }))
        .await;
    // should redirect to admin dashboard once logged in successfully
    assert_is_redirect_to(&response, "/admin/dashboard");

    // 2. Fill in the form
    let response = test_app
        .post_publish_issue(&serde_json::json!({
            "title": "Title",
            "text_content": "Text Content",
            "html_content": "<p>Html Content</p>",
            "idempotency_key": uuid::Uuid::new_v4().to_string(),
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // 3. Follow the redirect
    let html_page = test_app.get_admin_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // Arrange
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&test_app.email_server)
        .await;
    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Title",
        "text_content": "Text Content",
        "html_content": "<p>Html Content</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response1 = test_app.post_publish_issue(&newsletter_request_body);
    let response2 = test_app.post_publish_issue(&newsletter_request_body);
    // Submit two newsletter forms concurrently using `tokio::join!`
    let (response1, response2) = tokio::join!(response1, response2);
    // Assert
    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
}
