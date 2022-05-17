use {
    actix_web::{
        dev::Server,
        web::{self, Form},
        App, HttpResponse, HttpServer,
    },
    serde::Deserialize,
    std::net::TcpListener,
};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct FormData {
    name: String,
    email: String,
}

async fn subscribe(_form: Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscription", web::get().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
