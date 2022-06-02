use {
    crate::{
        authentication::{validate_credentials, AuthError, Credentials},
        routes::admin::dashboard::get_username,
        session_state::TypedSession,
        utils::{e500, see_other},
    },
    actix_web::{error::InternalError, web, HttpResponse},
    actix_web_flash_messages::FlashMessage,
    secrecy::ExposeSecret,
    secrecy::Secret,
    sqlx::PgPool,
    uuid::Uuid,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

async fn reject_anonymous_users(session: TypedSession) -> Result<Uuid, actix_web::Error> {
    match session.get_user_id().map_err(e500)? {
        Some(user_id) => Ok(user_id),
        None => {
            let response = see_other("/login");
            let e = anyhow::anyhow!("The user has not logged in");

            Err(InternalError::from_response(e, response).into())
        }
    }
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    // Check if login session is active
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    let user_id = reject_anonymous_users(session).await?;
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

    let username = get_username(user_id, &pool).await.map_err(e500)?;
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
    crate::authentication::change_password(user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();

    Ok(see_other("/admin/password"))
}
