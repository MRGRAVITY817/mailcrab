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
    let html_page = test_app.get_login_html().await;

    // Assert
    // 1. Check if error message is rendered in HTML page
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
    // 2. Check if it redirects after error
    assert_is_redirect_to(&response, "/login");
    // 3. Check again to see error message cookie deleted
    let html_page = test_app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    // 1. Login
    let login_body = serde_json::json!({
       "username": &test_app.test_user.username,
       "password": &test_app.test_user.password,
    });
    let response = test_app.post_login(&login_body).await;

    // Assert
    // 1. Check if redirects to dashboard
    assert_is_redirect_to(&response, "/admin/dashboard");
    // 2. Check if the dashboard page is rendered correctly
    let html_page = test_app.get_admin_dashboard().await;
    assert!(html_page.contains(&format!("Welcome {}", test_app.test_user.username)));
}
