use embassy_stm32::time::Hertz;
use embassy_stm32::{
    dma::NoDma,
    i2c::{Instance, InterruptHandler},
    i2c::{SclPin, SdaPin},
    interrupt::typelevel::Binding,
    Peripheral,
};

pub struct Bme680<'d, T: Instance> {
    i2c: embassy_stm32::i2c::I2c<'d, T>,
}

impl<T: Instance> Bme680<'_, T> {
    pub fn init<'d>(
        p: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Bme680<'d, T> {
        Bme680 {
            i2c: embassy_stm32::i2c::I2c::new(
                p,
                scl,
                sda,
                irq,
                NoDma,
                NoDma,
                Hertz(100_000),
                Default::default(),
            ),
        }
    }
}
