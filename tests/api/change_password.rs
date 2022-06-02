use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let response = test_app.get_change_password().await;

    // Assert
    // 1. Check if user is redirected to `login` page when not allowed to change password
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    // Assert
    // 1. Check if user is redirected to `login` page when not allowed to change password
    assert_is_redirect_to(&response, "/login");
}
