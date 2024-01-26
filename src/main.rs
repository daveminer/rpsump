use std::process::exit;

use rpsump::{config::Settings, middleware::telemetry, repository, startup::Application};

/// Start the application after loading settings, database, telemetry, and the RPi board.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_exit_handler();

    let settings = Settings::new();
    telemetry::init_tracer(&settings).expect("Could not initialize telemetry.");
    let gpio = rppal::gpio::Gpio::new().expect("Could not initialize GPIO.");
    // let repo = Box::new(
    //     repository::implementation(Some(settings.database_path))
    //         .await
    //         .expect("Could not initialize database."),
    // );

    // TODO: DB URI
    // let repo = match &settings.database_path {
    //     Some(database_path) => repository::implementation(database_path.clone())
    //         .context("Cannot create a repository")?,
    //     None => repository::mock()?,
    // };

    let repo = repository::implementation(Some(settings.database_path.clone()))
        .await
        .expect("Could not create repository.");

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
