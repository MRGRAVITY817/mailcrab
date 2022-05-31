use mailcrab::{
    configuration::get_config,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("mailcrab".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let app_config = get_config().expect("Failed to read configuration");
    let main_app = Application::build(app_config).await?;
    main_app.run_until_stopped().await?;

    Ok(())
}
