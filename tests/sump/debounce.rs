use rppal::gpio::Level;
use std::time::Duration;
use tokio::time::Instant;

use rpsump::hydro::debounce::Debouncer;

const DURATION: Duration = Duration::from_secs(1);

#[tokio::test]
async fn test_sensor_debouncer_new() {
    let now = Instant::now();
    let debouncer = SensorDebouncer::new(DURATION, Level::Low);

    // Check that the deadline is correctly set
    assert!(debouncer.get_deadline() > now + DURATION);
}

#[tokio::test]
async fn test_sensor_debouncer_reset_deadline() {
    let mut debouncer = SensorDebouncer::new(DURATION, Level::Low);

    let now = Instant::now();
    // Reset the deadline with a new reading of High
    debouncer.reset_deadline(Level::High);

    // Check that the deadline is correctly updated
    assert!(debouncer.get_deadline() > now + DURATION);
}

#[tokio::test]
async fn test_sensor_debouncer_sleep() {
    let debouncer = SensorDebouncer::new(DURATION, Level::Low);

    // Start sleeping and measure the time it takes
    let start_time = Instant::now();
    debouncer.sleep().await;
    let elapsed_time = start_time.elapsed();

    // Check that the elapsed time is close to the duration
    let tolerance = Duration::from_millis(15);
    assert!((elapsed_time >= DURATION - tolerance) && (elapsed_time <= DURATION + tolerance));
}
