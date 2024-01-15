use crate::hydro::{control::Output, Control};

struct PoolPump {
    speed1: Control,
    speed2: Control,
    speed3: Control,
    speed4: Control,
    current: PoolPumpSpeed,
}

enum PoolPumpSpeed {
    Low,
    Med,
    High,
    Max,
}

impl PoolPump {
    async fn off(&mut self) {
        vec![
            &mut self.speed1,
            &mut self.speed2,
            &mut self.speed3,
            &mut self.speed4,
        ]
        .iter_mut()
        .for_each(async move |speed_pin| {
            turn_off(speed_pin).await;
        });
    }

    async fn on(&mut self, speed: PoolPumpSpeed) {
        match speed {
            PoolPumpSpeed::Low => &mut self.speed1.on().await,
            PoolPumpSpeed::Med => &mut self.speed2.on(),
            PoolPumpSpeed::High => &mut self.speed3.on(),
            PoolPumpSpeed::Max => &mut self.speed4.on(),
        }
    }
}

async fn turn_off(speed_pin: &mut Control) {
    if speed_pin.is_on() {
        speed_pin.off().await;
    }
}
