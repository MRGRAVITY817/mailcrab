use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });
    let response = test_app.post_login(&login_body).await;
    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();
    let html_page = test_app.get_login_html().await;

    // Assert
    // 1. Check if the ephemeral error message is saved in cookie
    assert_eq!(flash_cookie.value(), "Authentication failed");
    // 2. Check if error message is rendered in HTML page
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
    // 3. Check if it redirects after error
    assert_is_redirect_to(&response, "/login");
    // 4. Check again to see error message cookie deleted
    let html_page = test_app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}
