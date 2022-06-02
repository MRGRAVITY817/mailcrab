use {
    crate::{
        authentication::{validate_credentials, AuthError, Credentials},
        routes::admin::dashboard::get_username,
        session_state::TypedSession,
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

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    // Check if login session is active
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    let user_id = session.get_user_id().map_err(e500)?;
    // If given `user_id` doesn't exists in redis session, redirect to `login` page
    if user_id.is_none() {
        return Ok(see_other("/login"));
    }
    let user_id = user_id.unwrap();
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
    todo!()
}
