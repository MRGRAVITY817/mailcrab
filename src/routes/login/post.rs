use {
    crate::{
        authentication::{validate_credentials, AuthError, Credentials},
        routes::error_chain_fmt,
    },
    actix_session::Session,
    actix_web::{error::InternalError, web, HttpResponse},
    actix_web_flash_messages::FlashMessage,
    reqwest::header::LOCATION,
    secrecy::Secret,
    sqlx::PgPool,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(
    skip(form, pool, session), 
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login_submit(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    session: Session,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    // Logs `username` input
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            // Log `user_id` if available
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            // Renew session token to avoid session fixation attacks.
            // For more info, https://acrossecurity.com/papers/session_fixation.pdf
            session.renew();
            session
                .insert("user_id", user_id)
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/admin/dashboard")) // Redirects to dashboard when post succeeds.
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            // To create ephemeral validation error, we use flash message.
            FlashMessage::error(e.to_string()).send();
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .finish();

            Err(InternalError::from_response(e, response))
        }
    }
}

/// If session management goes wrong, user will be redirected back to the "/login" page.
fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();

    InternalError::from_response(e, response)
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
