use actix_web::HttpResponse;
use reqwest::header::LOCATION;

pub async fn login() -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/")) // Redirects to home when post succeeds.
        .finish()
}
