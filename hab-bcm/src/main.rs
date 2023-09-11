#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod board;
mod device;

use board::{Board, OutdoorEnvSensor, StatusLed};

use embassy_executor::Spawner;

use defmt::println;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let board = Board::init();

    spawner.spawn(blink_status(board.status_led)).unwrap();
    spawner
        .spawn(monitor_outdoor_env(board.outdoor_env_sensor))
        .unwrap();
}

#[embassy_executor::task]
async fn blink_status(mut led: StatusLed) {
    loop {
        led.on();
        Timer::after(Duration::from_millis(300)).await;

        led.off();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn monitor_outdoor_env(mut sensor: OutdoorEnvSensor) {
    loop {
        let data = sensor.measure().await;

        println!("Temperature {}°C", data.temperature_celsius);
        println!("Pressure {}hPa", data.pressure_hpa);
        println!("Humidity {}%", data.humidity_percent);
        println!("Gas Resistance {}Ω", data.gas_resistance_ohm);

        Timer::after(Duration::from_secs(5)).await;
    }
}
