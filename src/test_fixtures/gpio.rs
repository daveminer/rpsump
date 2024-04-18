use crate::hydro::gpio::{Gpio, Level, MockGpio, MockInputPin, MockOutputPin, MockPin};
use crate::hydro::pool_pump::PoolPumpSpeed;
use crate::test_fixtures::settings::SETTINGS;
use mockall::*;

pub fn mock_control_gpio() -> impl Gpio {
    let mut mock_gpio = MockGpio::new();
    mock_gpio.expect_get().times(1).returning(|_| {
        let mut pin = MockPin::new();
        pin.expect_into_output_low()
            .times(1)
            .returning(|| Box::new(MockOutputPin::new()));
        Ok(Box::new(pin))
    });

    mock_gpio
}

pub fn mock_sensor_gpio() -> impl Gpio {
    let mut mock_gpio = MockGpio::new();
    mock_gpio.expect_get().times(1).returning(|_| {
        let mut pin = MockPin::new();
        pin.expect_into_input_pullup().times(1).returning(|| {
            let mut input_pin = MockInputPin::new();
            input_pin
                .expect_set_async_interrupt()
                .times(1)
                .returning(|_, _, _| Ok(()));
            input_pin.expect_read().times(1).returning(|| Level::Low);
            Box::new(input_pin)
        });
        Ok(Box::new(pin))
    });

    mock_gpio
}

pub fn build_mock_gpio() -> impl Gpio {
    let mut gpio = MockGpio::new();

    // Heater pin
    gpio = mock_heater(gpio, true);

    // Pool pump pins
    gpio = mock_pool_pump(gpio, PoolPumpSpeed::Max);

    // Sump pump pins
    gpio = mock_sump_pump(gpio, false, false, false);

    // Irrigation pins
    gpio = mock_irrigation_pump(gpio, false, false, None);

    gpio
}

pub fn mock_gpio_get(pins: Vec<u8>) -> Box<dyn Gpio> {
    let mut mock_gpio = MockGpio::new();
    for pin in pins {
        mock_gpio
            .expect_get()
            .with(predicate::eq(pin))
            .times(1)
            .returning(|_| Ok(Box::new(MockPin::new())));
    }
    Box::new(mock_gpio)
}

pub fn mock_input_pin_with_interrupt(is_on: bool, read_result: Level) -> Box<MockPin> {
    let mut mock_pin = MockPin::new();
    mock_pin.expect_into_input_pullup().returning(move || {
        let mut input_pin_stub = MockInputPin::new();
        input_pin_stub.expect_is_low().return_const(is_on);
        input_pin_stub
            .expect_set_async_interrupt()
            .returning(|_, _, _| Ok(()));
        input_pin_stub.expect_read().returning(move || read_result);
        Box::new(input_pin_stub)
    });
    Box::new(mock_pin)
}

pub fn mock_output_pin(is_on: bool) -> Box<MockPin> {
    let mut mock_pin = MockPin::new();
    mock_pin.expect_into_output_low().returning(move || {
        let mut output_pin_stub = MockOutputPin::new();
        output_pin_stub.expect_is_on().return_const(is_on);
        let _ = output_pin_stub.expect_off().return_const(());
        let _ = output_pin_stub.expect_on().return_const(());
        Box::new(output_pin_stub)
    });
    Box::new(mock_pin)
}

pub fn mock_heater(mut mock_gpio: MockGpio, is_on: bool) -> MockGpio {
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.heater.control_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(is_on)));

    mock_gpio
}

pub fn mock_irrigation_pump(
    mut mock_gpio: MockGpio,
    low_sensor_on: bool,
    running: bool,
    valve: Option<u8>,
) -> MockGpio {
    let mut valve1_open = false;
    let mut valve2_open = false;
    let mut valve3_open = false;
    let mut valve4_open = false;

    if running {
        let valve = valve.expect("Open valve is required if pump is running");
        if valve == 1 {
            valve1_open = true;
        } else if valve == 2 {
            valve2_open = true;
        } else if valve == 3 {
            valve3_open = true;
        } else if valve == 4 {
            valve4_open = true;
        } else {
            panic!("Invalid valve number");
        }
    }

    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.irrigation.low_sensor_pin))
        .times(1)
        .returning(move |_| Ok(mock_input_pin_with_interrupt(low_sensor_on, Level::Low)));
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.irrigation.pump_control_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(running)));
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.irrigation.valve_1_control_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(valve1_open)));
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.irrigation.valve_2_control_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(valve2_open)));
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.irrigation.valve_3_control_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(valve3_open)));
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.irrigation.valve_4_control_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(valve4_open)));

    mock_gpio
}

pub fn mock_pool_pump(mut mock_gpio: MockGpio, pump_speed: PoolPumpSpeed) -> MockGpio {
    let mut low_pin_on = false;
    let mut med_pin_on = false;
    let mut high_pin_on = false;
    let mut max_pin_on = false;

    if pump_speed == PoolPumpSpeed::Low {
        low_pin_on = true;
    } else if pump_speed == PoolPumpSpeed::Med {
        med_pin_on = true;
    } else if pump_speed == PoolPumpSpeed::High {
        high_pin_on = true;
    } else if pump_speed == PoolPumpSpeed::Max {
        max_pin_on = true;
    }

    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.pool_pump.low_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(low_pin_on)));

    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.pool_pump.med_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(med_pin_on)));

    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.pool_pump.high_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(high_pin_on)));

    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.pool_pump.max_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(max_pin_on)));

    mock_gpio
}

pub fn mock_sump_pump(
    mut mock_gpio: MockGpio,
    pump_on: bool,
    high_sensor_on: bool,
    low_sensor_on: bool,
) -> MockGpio {
    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.sump.pump_control_pin))
        .times(1)
        .returning(move |_| Ok(mock_output_pin(pump_on)));

    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.sump.high_sensor_pin))
        .times(1)
        .returning(move |_| Ok(mock_input_pin_with_interrupt(high_sensor_on, Level::Low)));

    mock_gpio
        .expect_get()
        .with(predicate::eq(SETTINGS.hydro.sump.low_sensor_pin))
        .times(1)
        .returning(move |_| Ok(mock_input_pin_with_interrupt(low_sensor_on, Level::Low)));

    mock_gpio
}
