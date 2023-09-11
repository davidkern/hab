use bme680::{FieldData, IIRFilterSize, OversamplingSetting, PowerMode, SettingsBuilder};
use core::time::Duration;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::{
    dma::NoDma,
    i2c::{Instance, InterruptHandler},
    i2c::{SclPin, SdaPin},
    interrupt::typelevel::Binding,
    Peripheral,
};
use embassy_time::Delay;
use embedded_hal::prelude::*;

pub struct Measurement {
    pub temperature_celsius: f32,
    pub pressure_hpa: f32,
    pub humidity_percent: f32,
    pub gas_resistance_ohm: u32,
}

impl From<FieldData> for Measurement {
    fn from(value: FieldData) -> Self {
        Measurement {
            temperature_celsius: value.temperature_celsius(),
            pressure_hpa: value.pressure_hpa(),
            humidity_percent: value.humidity_percent(),
            gas_resistance_ohm: value.gas_resistance_ohm(),
        }
    }
}
pub struct Bme680<'d, T: Instance> {
    delayer: Delay,
    dev: bme680::Bme680<I2c<'d, T>, Delay>,
}

impl<'d, T: Instance> Bme680<'d, T> {
    pub fn init(
        p: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Bme680<'d, T> {
        let i2c = embassy_stm32::i2c::I2c::new(
            p,
            scl,
            sda,
            irq,
            NoDma,
            NoDma,
            Hertz(100_000),
            Default::default(),
        );

        let mut delayer = Delay;
        let dev = bme680::Bme680::init(i2c, &mut delayer, bme680::I2CAddress::Primary).unwrap();

        Bme680 { delayer, dev }
    }

    pub async fn measure(&mut self) -> Measurement {
        let settings = SettingsBuilder::new()
            .with_humidity_oversampling(OversamplingSetting::OS2x)
            .with_pressure_oversampling(OversamplingSetting::OS4x)
            .with_temperature_oversampling(OversamplingSetting::OS8x)
            .with_temperature_filter(IIRFilterSize::Size3)
            .with_gas_measurement(Duration::from_millis(1500), 320, 25)
            .with_run_gas(true)
            .build();

        self.dev
            .set_sensor_settings(&mut self.delayer, settings)
            .unwrap();
        let profile_duration = self
            .dev
            .get_profile_dur(&settings.0)
            .expect("get profile duration");

        // Read sensor data
        self.dev
            .set_sensor_mode(&mut self.delayer, PowerMode::ForcedMode)
            .unwrap();
        self.delayer.delay_ms(profile_duration.as_millis() as u32);
        let (data, _state) = self.dev.get_sensor_data(&mut self.delayer).unwrap();

        data.into()
    }
}
