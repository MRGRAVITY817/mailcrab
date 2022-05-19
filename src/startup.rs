use {
    crate::routes::{health_check, subscribe},
    actix_web::{dev::Server, web, App, HttpServer},
    sqlx::PgPool,
    std::net::TcpListener,
    tracing_actix_web::TracingLogger,
};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
