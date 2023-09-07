use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    Peripheral,
};

pub struct Led<'d, T: Pin> {
    pin: Output<'d, T>,
}

// impl<T: OutputPin> Led for LedDevice<T> {
impl<T: Pin> Led<'_, T> {
    pub fn init<'d>(pin: impl Peripheral<P = T> + 'd) -> Led<'d, T> {
        Led {
            pin: Output::new(pin, Level::Low, Speed::Low),
        }
    }

    pub fn on(&mut self) {
        self.pin.set_high();
    }

    pub fn off(&mut self) {
        self.pin.set_low();
    }

    // pub fn set(&mut self, state: bool) {
    //     self.pin.set_state(state.into());
    // }
}
