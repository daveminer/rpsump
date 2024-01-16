use crate::hydro::gpio::{stub::pin, Level, MockGpio};
use mockall::*;

// pub fn mock_gpio_get(pin: u8) -> MockGpio {
//     let mut mock_gpio = MockGpio::new();
//     mock_gpio
//         .expect_get()
//         .with(predicate::eq(pin))
//         .times(1)
//         .returning(|_| {
//             Ok(Box::new(pin::PinStub {
//                 index: 0,
//                 level: Level::Low,
//             }))
//         });
//     mock_gpio
// }

pub fn mock_gpio_get(pins: Vec<u8>) -> MockGpio {
    let mut mock_gpio = MockGpio::new();
    for pin in pins {
        mock_gpio
            .expect_get()
            .with(predicate::eq(pin))
            .times(1)
            .returning(|_| {
                Ok(Box::new(pin::PinStub {
                    index: 0,
                    level: Level::Low,
                }))
            });
    }
    mock_gpio
}
