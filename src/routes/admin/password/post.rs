use {
    crate::{
        session_state::TypedSession,
        utils::{e500, see_other},
    },
    actix_web::{web, HttpResponse},
    actix_web_flash_messages::FlashMessage,
    secrecy::ExposeSecret,
    secrecy::Secret,
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
) -> Result<HttpResponse, actix_web::Error> {
    // Check if login session is active
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
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
    todo!()
}
