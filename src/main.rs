use std::process::exit;

use rpsump::config::Settings;
use rpsump::database::new_pool;
use rpsump::middleware::telemetry;
use rpsump::startup::Application;

/// Start the application after loading settings, database, telemetry, and the RPi board.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_exit_handler();

    let settings = Settings::new();
    let db_pool = new_pool(&settings.database_url).expect("Could not create database pool.");
    telemetry::init_tracer(&settings).expect("Could not initialize telemetry.");
    let gpio = rppal::gpio::Gpio::new().expect("Could not initialize GPIO.");
    let application = Application::build(settings, &db_pool, gpio);

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
