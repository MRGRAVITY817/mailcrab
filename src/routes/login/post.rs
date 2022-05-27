use actix_web::{web, HttpResponse};
use reqwest::header::LOCATION;
use secrecy::Secret;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

pub async fn login_submit(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/")) // Redirects to home when post succeeds.
        .finish()
}
