use {
    crate::helpers::{assert_is_redirect_to, spawn_app},
    uuid::Uuid,
};

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

#[tokio::test]
async fn new_password_fields_must_match() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // Act
    // 1. Login
    test_app
        .post_login(&serde_json::json!({
            "username": &test_app.test_user.username,
            "password": &test_app.test_user.password,
        }))
        .await;
    // 2. Try to change the password
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": &test_app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &another_new_password,
        }))
        .await;

    // Assert
    // 1. Check if mismatching the new passwords will redirect to `admin/password` page
    assert_is_redirect_to(&response, "/admin/password");
    // 2. Follow the redirect, check the html page
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - the field values must match.</i></p>"
    ));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_current_password = Uuid::new_v4().to_string();

    // Act
    // 1. Login
    test_app
        .post_login(&serde_json::json!({
            "username": &test_app.test_user.username,
            "password": &test_app.test_user.password,
        }))
        .await;
    // 2. Try to change the password
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": &wrong_current_password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    // Assert
    // 1. If current password is invalid, server will redirect user to `admin/password` page
    assert_is_redirect_to(&response, "/admin/password");
    // 2. Follow the redirect, check the html page
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn new_password_should_be_longer_than_12_chars() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = "c".repeat(11);

    // Act
    // 1. Login
    test_app
        .post_login(&serde_json::json!({
            "username": &test_app.test_user.username,
            "password": &test_app.test_user.password,
        }))
        .await;
    // 2. Try to change the password
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": &test_app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    // Assert
    // 1. If current password is invalid, server will redirect user to `admin/password` page
    assert_is_redirect_to(&response, "/admin/password");
    // 2. Follow the redirect, check the html page
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>Password should be longer that 12 chars and shorter than 128 chars.</i></p>"
    ));
}

#[tokio::test]
async fn new_password_should_be_shorter_than_128_chars() {
    // Arrange
    let test_app = spawn_app().await;
    let new_password = "c".repeat(128);

    // Act
    // 1. Login
    test_app
        .post_login(&serde_json::json!({
            "username": &test_app.test_user.username,
            "password": &test_app.test_user.password,
        }))
        .await;
    // 2. Try to change the password
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": &test_app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    // Assert
    // 1. If current password is invalid, server will redirect user to `admin/password` page
    assert_is_redirect_to(&response, "/admin/password");
    // 2. Follow the redirect, check the html page
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>Password should be longer that 12 chars and shorter than 128 chars.</i></p>"
    ));
}
