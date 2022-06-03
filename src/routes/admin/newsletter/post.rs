use actix_web::HttpResponse;

pub async fn send_newsletter() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}
