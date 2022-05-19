use {
    crate::{
        email_client::EmailClient,
        routes::{health_check, subscribe},
    },
    actix_web::{dev::Server, web, App, HttpServer},
    sqlx::PgPool,
    std::net::TcpListener,
    tracing_actix_web::TracingLogger,
};

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // `web::Data` is basically `Arc`, which will safely share the app state across threads
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
