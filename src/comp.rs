use crate::device::Chsc6x;
use embedded_hal::digital::{ InputPin, OutputPin };
use embedded_hal_async::{ delay::DelayNs, digital::Wait, i2c::I2c };
use crate::error::{Error, DeviceError};

struct FirmwareInfo {
    chsc6x_cfg_version: u16,  //customer read 
    chsc6x_boot_version: u16, //customer read 
    chsc6x_vendor_id: u16,    //customer read 
    chsc6x_project_id: u16,   //customer read 
    chsc6x_chip_id: u16,      //customer read 
    chsc6x_chip_type: u16,    //customer read 
    chsc6x_rpt_lcd_x: u16,    //customer read must after chsc6x_get_chip_info
    chsc6x_rpt_lcd_y: u16,    //customer read must after chsc6x_get_chip_info
    chsc6x_max_pt_num: u16,   //customer read must after chsc6x_get_chip_info
}

impl<I2C, INT, RST, TMR: DelayNs, EC, EI, EO> Chsc6x<I2C, INT, RST, TMR> 
where
    I2C: I2c<Error = EC>,
    INT: Wait<Error = EI>,
    RST: OutputPin<Error = EO>,
{    
    pub async fn upgrade(&mut self) -> Result<FirmwareInfo, Error<EC, EI, EO>> {
        Ok(())
    }

    pub async fn detect(&mut self) -> Result<FirmwareInfo, Error<EC, EI, EO>> {
        Ok(())
    }

    pub async fn new_detected(
        i2c: I2C,
        interrupt_pin: INT,
        reset_pin: RST,
        delay: TMR,
    ) -> Result<Self, Error<EC, EI, EO>> {
        let mut device = Self::new(i2c, interrupt_pin, reset_pin, delay).await?;
        let mut result = match device.detect().await {
            Ok(_) => return Ok(device),
            Err(e) => e,
        };
        for i in 0..2 {
            match device.detect().await {
                Ok(_)=> return Ok(device),
                Err(e) => result = e,
            }
        }
        Err(result)
    }
}
