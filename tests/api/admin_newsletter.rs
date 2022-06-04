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
