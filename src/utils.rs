use {actix_web::HttpResponse, reqwest::header::LOCATION};

/// Return `InternalServerError` for `actix_web::Error` type
pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

/// Returns `HttpResponse` that redirects to other page
pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
