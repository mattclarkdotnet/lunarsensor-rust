use anyhow::{anyhow, Result};
use rppal::i2c::I2c;
use std::sync::Mutex;
use std::{thread, time};

const VEML6030_ADDRESS: u16 = 0x10;
const VEML6030_REG_CONF: u8 = 0x00;
const VEML6030_REG_ALS: u8 = 0x04;
const VEML6030_DEFAULT_SETTINGS: u8 = 0x00; // gain:1x, integration 100ms, persistence 1, disable interrupt
const VEML6030_CONVERSION_FACTOR: f32 = 0.0576;

pub type LockableSensor = Mutex<I2c>;

// Using the anyhow crate as my Rust isn't good enough to figure out the threading problems that
// come with I2c errors not being Send or Sync

pub fn setup_sensor() -> Result<LockableSensor> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(VEML6030_ADDRESS)?;
    i2c.block_write(VEML6030_REG_CONF, &[VEML6030_DEFAULT_SETTINGS])?;
    thread::sleep(time::Duration::from_millis(250)); // let the device settle
    return Ok(LockableSensor::new(i2c));
}

pub fn read_sensor(lockable_sensor: &LockableSensor) -> Result<f32> {
    let mut data = [0u8; 2];
    let locked_sensor = match lockable_sensor.lock() {
        Ok(v) => v,
        Err(_e) => return Err(anyhow!("Failed to lock sensor")),
    };
    return match locked_sensor.block_read(VEML6030_REG_ALS, &mut data) {
        Ok(_v) => {
            Ok((((data[1] as u16) << 8) | data[0] as u16) as f32 * VEML6030_CONVERSION_FACTOR)
        }
        Err(_e) => Err(anyhow!("Failed to read sensor")),
    };
}
