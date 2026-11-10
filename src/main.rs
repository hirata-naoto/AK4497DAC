#![no_std]
#![no_main]

mod ak4497;

use ak4497::{Ak4497, Dem, Dif};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, Config as I2cConfig, InterruptHandler};
use embassy_rp::peripherals::I2C0;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

// Pin assignment (Raspberry Pi Pico):
//   GP4 = I2C0 SDA -> AK4497 SDA
//   GP5 = I2C0 SCL -> AK4497 SCL
//   GP6 = PDN (active low, power-down / reset)
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut cfg = I2cConfig::default();
    cfg.frequency = 400_000;
    let i2c = i2c::I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, cfg);
    let pdn = Output::new(p.PIN_6, Level::Low);

    let mut dac = Ak4497::new(i2c, pdn, 0x10);

    unwrap!(dac.power_on().await);
    unwrap!(dac.set_format(Dif::I2s32).await);
    unwrap!(dac.set_deemphasis(Dem::Off).await);
    unwrap!(dac.set_volume(0xFF, 0xFF).await);
    unwrap!(dac.set_mute(false).await);
    unwrap!(dac.start().await);
    info!("AK4497 initialised");

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
