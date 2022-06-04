use crate::helpers::{assert_is_redirect_to, spawn_app};

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
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // 3. Follow the redirect
    let html_page = test_app.get_admin_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
}
