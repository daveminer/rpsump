struct PoolPump {
    speed1: Control,
    speed2: Control,
    speed3: Control,
    speed4: Control,
    current: PoolPumpSpeed,
}

enum PoolPumpSpeed {
    Off,
}

fn off() {}

fn on() {}
