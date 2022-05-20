use mailcrab::{
    configuration::get_config,
    startup::build,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("mailcrab".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let app_config = get_config().expect("Failed to read configuration");
    let server = build(app_config).await?;

    server.await?;
    Ok(())
}
