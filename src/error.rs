#[derive(Debug, Clone, Copy)]
pub enum Error<I2C, IN, OUT> {
    I2c(I2C),
    Wait(IN),
    OutputPin(OUT),
    Device(DeviceError),
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceError {
    InvalidTouchFlag(u8)
}
