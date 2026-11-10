//! Minimal AK4497 driver over async I2C.

use embassy_rp::gpio::Output;
use embassy_rp::i2c::{Async, Error, I2c, Instance};
use embassy_time::{Duration, Timer};

const REG_CONTROL1: u8 = 0x00;
const REG_CONTROL2: u8 = 0x01;
const REG_LCH_ATT: u8 = 0x03;
const REG_RCH_ATT: u8 = 0x04;

const CTRL1_RSTN: u8 = 1 << 0;
const CTRL2_SMUTE: u8 = 1 << 0;

/// Audio interface format (DIF[2:0], Control 1 bits 3:1).
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Dif {
    Lsb16 = 0b000,
    Lsb20 = 0b001,
    Msb32 = 0b010,
    I2s32 = 0b011,
    Lsb24 = 0b100,
    Lsb32 = 0b101,
}

/// De-emphasis (DEM[1:0], Control 2 bits 2:1).
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Dem {
    Khz44_1 = 0b00,
    Off = 0b01,
    Khz48 = 0b10,
    Khz32 = 0b11,
}

pub struct Ak4497<'d, T: Instance> {
    i2c: I2c<'d, T, Async>,
    pdn: Output<'d>,
    addr: u8,
}

impl<'d, T: Instance> Ak4497<'d, T> {
    /// `addr` is the 7-bit address (0x10..=0x13 depending on CAD pins).
    pub fn new(i2c: I2c<'d, T, Async>, pdn: Output<'d>, addr: u8) -> Self {
        Self { i2c, pdn, addr }
    }

    /// Toggle PDN and wait for the device to become ready.
    pub async fn power_on(&mut self) -> Result<(), Error> {
        self.pdn.set_low();
        Timer::after(Duration::from_millis(2)).await;
        self.pdn.set_high();
        Timer::after(Duration::from_millis(5)).await;
        Ok(())
    }

    pub async fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Error> {
        self.i2c.write_async(self.addr, [reg, val]).await
    }

    pub async fn read_reg(&mut self, reg: u8) -> Result<u8, Error> {
        let mut buf = [0u8; 1];
        self.i2c.write_read_async(self.addr, [reg], &mut buf).await?;
        Ok(buf[0])
    }

    async fn modify(&mut self, reg: u8, mask: u8, val: u8) -> Result<(), Error> {
        let cur = self.read_reg(reg).await?;
        self.write_reg(reg, (cur & !mask) | (val & mask)).await
    }

    pub async fn set_format(&mut self, dif: Dif) -> Result<(), Error> {
        self.modify(REG_CONTROL1, 0b0000_1110, (dif as u8) << 1).await
    }

    pub async fn set_deemphasis(&mut self, dem: Dem) -> Result<(), Error> {
        self.modify(REG_CONTROL2, 0b0000_0110, (dem as u8) << 1).await
    }

    /// 0xFF = 0 dB, 0.5 dB/step, 0x00 = mute.
    pub async fn set_volume(&mut self, left: u8, right: u8) -> Result<(), Error> {
        self.write_reg(REG_LCH_ATT, left).await?;
        self.write_reg(REG_RCH_ATT, right).await
    }

    pub async fn set_mute(&mut self, mute: bool) -> Result<(), Error> {
        self.modify(REG_CONTROL2, CTRL2_SMUTE, mute as u8).await
    }

    /// Release the internal reset (RSTN=1) to start conversion.
    pub async fn start(&mut self) -> Result<(), Error> {
        self.modify(REG_CONTROL1, CTRL1_RSTN, CTRL1_RSTN).await
    }
}
