use std::time::Duration;
use tokio::time::Instant;

use rpsump::hydro::{debounce::Debouncer, gpio::Level, signal::Message};

const DURATION: Duration = Duration::from_secs(1);

#[tokio::test]
async fn test_sensor_debouncer_new() {
    let now = Instant::now();
    let mpsc = tokio::sync::mpsc::channel(32);
    let debouncer = Debouncer::new(Level::Low, DURATION, Message::SumpFull, mpsc.0);

    // Check that the deadline is correctly set
    assert!(debouncer.get_deadline() > now + DURATION);
}

#[tokio::test]
async fn test_sensor_debouncer_reset_deadline() {
    let mpsc = tokio::sync::mpsc::channel(32);
    let mut debouncer = Debouncer::new(Level::Low, DURATION, Message::SumpFull, mpsc.0);

    let now = Instant::now();
    // Reset the deadline with a new reading of High
    debouncer.reset_deadline(Level::High);

    // Check that the deadline is correctly updated
    assert!(debouncer.get_deadline() > now + DURATION);
}

#[tokio::test]
async fn test_sensor_debouncer_sleep() {
    let mpsc = tokio::sync::mpsc::channel(32);
    let debouncer = Debouncer::new(Level::Low, DURATION, Message::SumpFull, mpsc.0);

    // Start sleeping and measure the time it takes
    let start_time = Instant::now();
    debouncer.sleep().await;
    let elapsed_time = start_time.elapsed();

    // Check that the elapsed time is close to the duration
    let tolerance = Duration::from_millis(15);
    assert!((elapsed_time >= DURATION - tolerance) && (elapsed_time <= DURATION + tolerance));
}
