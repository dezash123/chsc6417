use embedded_hal::digital::OutputPin;
use embedded_hal_async::{ delay::DelayNs, digital::Wait, i2c::I2c };
use crate::error::{Error, DeviceError};

pub const I2C_ADR: u8 = 0x5c; //8bit 

// #define CHSC6X_MAX_POINTS_NUM     (1)
pub const MAX_X: u16 = 370;
pub const MAX_Y: u16 = 370;
pub const RESET_TIME_MS: u32 = 30;
pub const SUSPEND_CODE: [u8; 2] = [0xa5, 0x03];
pub const DBCHECK: [u8; 2] = [0xD0, 0x01];
pub const PALM_CHECK: [u8; 2] = [0xD1, 0x01];

// /*MACRO SWITCH for driver update TP FW */
// #define CHSC6X_AUTO_UPGRADE           (0)
// 
// /*MACRO SWITCH for multi TP_VENDOR Compatible update TP FW */
// #define CHSC6X_MUL_VENDOR_UPGRADE     (0)
// 
// #define MAX_IIC_WR_LEN          (8)
// #define MAX_IIC_RD_LEN          (16)

#[derive(Debug, Copy, Clone)]
pub enum TouchFlag {
  Down = 0,
  Up = 1,
  Contact = 2,
}

#[derive(Debug, Copy, Clone)]
pub struct TouchEvent {
  pub x: u16, /*x coordinate */
  pub y: u16, /*y coordinate */
  pub flag: TouchFlag, /* touch event flag: 0 -- down; 1-- up; 2 -- contact */
  pub id: bool,   /*touch ID */
}

pub struct Chsc6x<I2C: I2c, INT: Wait, RST: OutputPin, TMR: DelayNs> {
    pub i2c: I2C,
    pub interrupt_pin: INT,
    pub reset_pin: RST,
    pub delay: TMR,
    pub suspended: bool,
}

impl<I2C, INT, RST, TMR: DelayNs, EC, EI, EO> Chsc6x<I2C, INT, RST, TMR> 
where
    I2C: I2c<Error = EC>,
    INT: Wait<Error = EI>,
    RST: OutputPin<Error = EO>,
{    
    #[inline]
    async fn i2c_read(&mut self, buffer: &mut [u8]) -> Result<(), Error<EC, EI, EO>> {
        self.i2c.read(I2C_ADR, buffer).await.map_err(Error::<EC, EI, EO>::I2c)
    }

    #[inline]
    async fn i2c_write(&mut self, buffer: &[u8]) -> Result<(), Error<EC, EI, EO>> {
        self.i2c.write(I2C_ADR, buffer).await.map_err(Error::<EC, EI, EO>::I2c)
    }

    async fn active_reset(&mut self) -> Result<(), Error<EC, EI, EO>> {
        self.reset_pin.set_low().map_err(Error::<EC, EI, EO>::OutputPin)?;
        self.delay.delay_ms(RESET_TIME_MS).await;
        self.reset_pin.set_high().map_err(Error::<EC, EI, EO>::OutputPin)?;
        self.suspended = false;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), Error<EC, EI, EO>> {
        self.active_reset().await?;
        self.delay.delay_ms(RESET_TIME_MS).await;
        Ok(())
    }

    pub async fn read_last(&mut self) -> Result<TouchEvent, Error<EC, EI, EO>> {
        let mut data = [0u8; 3];
        self.i2c_read(&mut data).await?;
        Ok(TouchEvent {
            x: u16::from_be_bytes([(data[0] >> 6) & 1, data[1]]),
            y: u16::from_be_bytes([(data[0] >> 7) & 1, data[2]]),
            flag: match data[0] & 0b110000 {
                0 => TouchFlag::Down,
                16 => TouchFlag::Up,
                32 => TouchFlag::Contact,
                _ => return Err(Error::Device(DeviceError::InvalidTouchFlag((data[0] << 4) & 3))),
            },
            id: data[0] & 0b100 != 0,
        })
    }

    pub async fn suspend(&mut self) -> Result<(), Error<EC, EI, EO>> {
        if !self.suspended {
            self.reset().await?;
            self.i2c_write(&SUSPEND_CODE).await?;
            self.suspended = true;
        }
        Ok(())
    }
    pub async fn resume(&mut self) -> Result<(), Error<EC, EI, EO>> {
        if self.suspended {
            self.reset().await?;
        }
        Ok(())
    }

    // debounce?
    pub async fn dbcheck(&mut self) -> Result<(), Error<EC, EI, EO>> {
        self.reset().await?;
        self.i2c_write(&DBCHECK).await?;
        Ok(())
    }

    // palm touch?
    pub async fn palm_check(&mut self) -> Result<(), Error<EC, EI, EO>> {
        self.reset().await?;
        self.i2c_write(&PALM_CHECK).await?;
        Ok(())
    }

    pub async fn wait_on_touch(&mut self) -> Result<TouchEvent, Error<EC, EI, EO>> {
        self.interrupt_pin.wait_for_rising_edge().await.map_err(Error::<EC, EI, EO>::Wait)?;
        self.read_last().await
    }

    pub async fn new(
        i2c: I2C,
        interrupt_pin: INT,
        reset_pin: RST,
        delay: TMR,
    ) -> Result<Self, Error<EC, EI, EO>> {
        let mut device = Self {
            i2c,
            interrupt_pin,
            reset_pin,
            delay,
            suspended: true,
        };
        device.resume().await?;
        Ok(device)
    }
}
