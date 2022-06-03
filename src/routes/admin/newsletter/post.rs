use actix_web::HttpResponse;

pub async fn publish_issue() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}
