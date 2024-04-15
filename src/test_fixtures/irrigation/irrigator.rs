use rstest::fixture;

use crate::{
    hydro::{
        control::Control, gpio::Trigger, irrigator::Irrigator, sensor::Sensor, signal::Message,
    },
    test_fixtures::gpio::mock_gpio_get,
};

#[fixture]
pub fn irrigator() -> Irrigator {
    let mock_gpio = mock_gpio_get(vec![1, 2, 3, 4, 5, 6]);
    let (tx, _rx) = tokio::sync::mpsc::channel(32);

    let pump = Control::new("Pump".to_string(), 1, &mock_gpio).unwrap();
    let valve1 = Control::new("Valve1".to_string(), 2, &mock_gpio).unwrap();
    let valve2 = Control::new("Valve2".to_string(), 3, &mock_gpio).unwrap();
    let valve3 = Control::new("Valve3".to_string(), 4, &mock_gpio).unwrap();
    let valve4 = Control::new("Valve4".to_string(), 5, &mock_gpio).unwrap();

    let low_sensor = Sensor::new(Message::SumpEmpty, 6, &mock_gpio, Trigger::Both, &tx).unwrap();

    Irrigator {
        low_sensor,
        pump,
        valve1,
        valve2,
        valve3,
        valve4,
    }
}
