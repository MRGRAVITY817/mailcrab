use {
    crate::{
        authentication::{validate_credentials, AuthError, Credentials, UserId},
        routes::admin::dashboard::get_username,
        utils::{e500, see_other},
    },
    actix_web::{web, HttpResponse},
    actix_web_flash_messages::FlashMessage,
    secrecy::ExposeSecret,
    secrecy::Secret,
    sqlx::PgPool,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

/// Change user password
pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();

    // Check if new password is too short or too long (should be > 12 && < 128 chars)
    if form.new_password.expose_secret().chars().count().le(&12)
        || form.new_password.expose_secret().chars().count().ge(&128)
    {
        FlashMessage::error("Password should be longer that 12 chars and shorter than 128 chars.")
            .send();
        return Ok(see_other("/admin/password"));
    }

    // Check if two input data for new password is equal
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        // If not, send flash message to indicate error to user on page.
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(*user_id, &pool).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };

    // Check if current password is valid
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e)),
        };
    }

    // Change password
    crate::authentication::change_password(*user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();

    Ok(see_other("/admin/password"))
}
