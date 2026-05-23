use mockall::predicate;

use crate::hydro::gpio::MockGpio;
use crate::test_fixtures::gpio::mock_output_pin;
use crate::test_fixtures::settings::SETTINGS;

pub fn mock_garden(mut mock_gpio: MockGpio, solenoid_on: bool) -> MockGpio {
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.garden.solenoid_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(solenoid_on)));
    mock_gpio
}
