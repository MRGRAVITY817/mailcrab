use mailcrab::{
    configuration::get_config,
    issue_delivery_worker::run_worker_until_stopped,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("mailcrab".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let app_config = get_config().expect("Failed to read configuration");
    let main_app = Application::build(app_config.clone()).await?;

    // Create tasks to be in separate threads
    let app_task = tokio::spawn(main_app.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(app_config));

    // Run both tasks concurrently / in parallel
    // this will run until one of the two tasks completes or errors out
    tokio::select! {
        _ = app_task => {},
        _ = worker_task => {},
    };

    Ok(())
}
