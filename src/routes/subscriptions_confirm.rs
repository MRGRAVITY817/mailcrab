use actix_web::web;

use {actix_web::HttpResponse, serde::Deserialize};

#[derive(Deserialize)]
pub struct Parameters {
    subscriptions_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
