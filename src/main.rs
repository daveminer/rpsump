use std::process::exit;

use rpsump::{config::Settings, middleware::telemetry, repository, startup::Application};

/// Start the application after loading settings, database, telemetry, and the RPi board.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_exit_handler();

    // Application config
    let settings = Settings::new();

    // Observability
    telemetry::init_tracer(&settings).expect("Could not initialize telemetry.");

    // Raspberry Pi
    let gpio = rppal::gpio::Gpio::new().expect("Could not initialize GPIO.");

    // TODO: DB URI
    // Database
    let repo = repository::implementation(Some(settings.database_path.clone()))
        .await
        .expect("Could not create repository.");

    // Application
    let application = Application::build(settings, &gpio, repo);

    application.run_until_stopped().await?;

    Ok(())
}

// actix-web will handle signals to exit, but doesn't offer a hook to customize it.
fn init_exit_handler() {
    ctrlc::set_handler(move || {
        // Ensure all spans have been reported.
        opentelemetry::global::shutdown_tracer_provider();

        exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}
