use sqlx::{Connection, PgConnection};

use {
    mailcrab::{configuration::get_configuration, startup::run},
    std::net::TcpListener,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app_config = get_configuration().expect("Failed to read configuration");
    let connection = PgConnection::connect(&app_config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", app_config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
