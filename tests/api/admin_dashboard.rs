use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let response = test_app.get_admin_dashboard().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    // Arrange
    let test_app = spawn_app().await;

    // Act & Assert
    // 1. Login
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password,
    });
    let response = test_app.post_login(&login_body).await;
    // should redirect to dashboard when login success
    assert_is_redirect_to(&response, "/admin/dashboard");

    // 2. Follow the redirect
    let html_page = test_app.get_admin_dashboard_html().await;
    // Should render welcoming sentence
    assert!(html_page.contains(&format!("Welcome {}", test_app.test_user.username)));

    // 3. Logout
    let response = test_app.post_logout().await;
    // should redirect to `login` page when user logged out
    assert_is_redirect_to(&response, "/login");

    // 4. Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    // 5. Try to load admin dashboard
    let response = test_app.get_admin_dashboard().await;
    // Since logged out, it should redirect to `login` page
    assert_is_redirect_to(&response, "/login");
}
