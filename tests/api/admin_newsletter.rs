use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn should_be_logged_in_to_send_newsletter() {
    // Arrange
    let test_app = spawn_app().await;
    // Act
    let response = test_app.get_admin_newsletter().await;
    // Assert
    // Redirects to `login` when user is not logged in
    assert_is_redirect_to(&response, "/login");
}
