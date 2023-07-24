#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub use esp32s2_hal as hal;

use embassy_executor::Executor;
use embassy_executor::_export::StaticCell;
use esp_backtrace as _;
use esp_hal_common::clock::CpuClock;
use esp_hal_common::Rng;
use esp_println::logger::init_logger;
use esp_wifi::wifi::WifiMode;
use esp_wifi::EspWifiInitFor;
use hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, Delay, Rtc, IO,
};

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

type RedLed = hal::gpio::GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 13>;

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);

    // Configure clocks, timers and peripherals
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let mut peripheral_clock_control = system.peripheral_clock_control;
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    rtc.rwdt.disable();
    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, &mut peripheral_clock_control).timer0;

    let mut rng = Rng::new(peripherals.RNG);
    // NOTE: this won't be as random as it should be... wifi needs to be on first.
    // But if wifi is on, then we've given up access to the hardware RNG.
    let net_stack_seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Configure wifi
    let wifi_init = esp_wifi::initialize(
        EspWifiInitFor::Wifi,
        timer,
        rng,
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    let wifi = peripherals.RADIO.split();
    let (wifi_interface, _wifi_controller) =
        esp_wifi::wifi::new_with_mode(&wifi_init, wifi, WifiMode::Sta);

    let dhcp_config = embassy_net::Config::dhcpv4(Default::default());

    // Configure network stack
    let _net_stack = {
        &*singleton!(embassy_net::Stack::new(
            wifi_interface,
            dhcp_config,
            singleton!(embassy_net::StackResources::<3>::new()),
            net_stack_seed
        ))
    };

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let red_led = io.pins.gpio13.into_push_pull_output();
    let status_delay = Delay::new(&clocks);

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(status_led(red_led, status_delay)).ok();
    })
}

#[embassy_executor::task]
async fn status_led(mut led: RedLed, mut delay: Delay) {
    led.set_high().unwrap();

    loop {
        delay.delay_ms(500u32);
        led.toggle().unwrap();
    }
}
