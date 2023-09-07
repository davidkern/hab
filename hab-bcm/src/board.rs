use embassy_stm32::{
    gpio::Pin,
    peripherals::{I2C1, PD12, self}, bind_interrupts, i2c,
};

use crate::device::{led::Led, bme680::Bme680};

pub type StatusLed = Led<'static, PD12>;
pub type OutdoorEnvSensor = Bme680<'static, peripherals::I2C1>;

bind_interrupts!(struct Irq {
    I2C1_EV => i2c::InterruptHandler<peripherals::I2C1>;
});

pub struct Board {
    pub status_led: StatusLed,
    pub outdoor_env_sensor: OutdoorEnvSensor,
}

impl Board {
    pub fn init() -> Self {
        let p = embassy_stm32::init(Default::default());

        Board {
            status_led: Led::init(p.PD12),
            outdoor_env_sensor: Bme680::init(p.I2C1, p.PB6, p.PB7, Irq),
        }
    }
}
