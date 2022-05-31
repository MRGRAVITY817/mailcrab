
use {
    crate::{
        authentication::{validate_credentials, AuthError, Credentials},
        routes::error_chain_fmt,
    },
    actix_web::{error::InternalError, web, HttpResponse, cookie::Cookie},
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
    skip(form, pool), 
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login_submit(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
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

            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/")) // Redirects to home when post succeeds.
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                // To create ephemeral validation error, we use cookie.
                .cookie(Cookie::new("_flash", e.to_string()))
                .finish();

            Err(InternalError::from_response(e, response))
        }
    }
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
