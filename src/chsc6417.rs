use embedded_hal::digital::{ InputPin, OutputPin };
use embedded_hal_async::{ delay::DelayNs, digital::Wait, i2c::I2c };
use anyhow::Result;
use core::error::Error;

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


// struct ts_event {
//     unsigned short x; /*x coordinate */
//     unsigned short y; /*y coordinate */
//     int flag; /* touch event flag: 0 -- down; 1-- up; 2 -- contact */
//     int id;   /*touch ID */
// };
pub struct TouchEvent {
    x: u16,
    y: u16,
    flag: TouchFlag,
    id: bool,
}

#[repr(u8)]
pub enum TouchFlag {
    Down,
    Up,
    Contact,
    Invalid(u8),
}

// events[0].x = (unsigned short)(((read_buf[0] & 0x40) >> 6) << 8) | (unsigned short)read_buf[1];
// events[0].y = (unsigned short)(((read_buf[0] & 0x80) >> 7) << 8) | (unsigned short)read_buf[2];
// events[0].flag= (read_buf[0] >> 4) & 0x03;
// events[0].id = (read_buf[0] >>2) & 0x01;

impl From<u8> for TouchFlag {
    fn from(value: u8) -> Self {
        match value & 0b110000 {
            0 => Self::Down,
            const { 1 << 4 } => Self::Up,
            const { 2 << 4 } => Self::Contact,
            _ => Self::Invalid((value << 4) & 3),
        }
    }
}

impl From<[u8; 3]> for TouchEvent {
    fn from(data: [u8; 3]) -> Self {
        Self {
            x: u16::from_be_bytes([(data[0] >> 6) & 1, data[1]]),
            y: u16::from_be_bytes([(data[0] >> 7) & 1, data[2]]),
            flag: TouchFlag::from(data[0]),
            id: data[0] & 0b100 != 0,
        }
    }
}

pub struct Chsc6417<I2C, INT, RST, TMR> {
    i2c: I2C,
    interrupt_pin: INT,
    reset_pin: RST,
    delay: TMR,
    suspend: bool,
}

impl<I2C: I2c, INT: Wait, RST: OutputPin, TMR: DelayNs> Chsc6417<I2C, INT, RST, TMR>
    where
        // ideally these can be relaxed one day...
        I2C::Error: Sync + Error + Send + 'static,
        INT::Error: Sync + Error + Send + 'static,
        RST::Error: Sync + Error + Send + 'static,
{
    async fn active_reset(&mut self) -> Result<()> {
        self.reset_pin.set_low()?;
        self.delay.delay_ms(RESET_TIME_MS).await;
        self.reset_pin.set_high()?;
        self.suspend = false;
        Ok(())
    }
    pub async fn reset(&mut self) -> Result<()> {
        self.active_reset().await?;
        self.delay.delay_ms(RESET_TIME_MS).await;
        Ok(())
    }

    pub async fn read_last(&mut self) -> Result<TouchEvent> {
        let mut buffer = [0u8; 3];
        self.i2c.read(I2C_ADR, &mut buffer).await?;
        Ok(TouchEvent::from(buffer))
    }
    pub async fn suspend(&mut self) -> Result<()> {
        if !self.suspend {
            self.reset().await?;
            self.i2c.write(I2C_ADR, &SUSPEND_CODE).await?;
            self.suspend = true;
        }
        Ok(())
    }
    pub async fn resume(&mut self) -> Result<()> {
        if self.suspend {
            self.reset().await?;
        }
        Ok(())
    }

    // debounce?
    pub async fn dbcheck(&mut self) -> Result<()> {
        self.reset().await?;
        self.i2c.write(I2C_ADR, &DBCHECK).await?;
        Ok(())
    }

    // palm touch?
    pub async fn palm_check(&mut self) -> Result<()> {
        self.reset().await?;
        self.i2c.write(I2C_ADR, &PALM_CHECK).await?;
        Ok(())
    }

    pub async fn wait_on_touch(&mut self) -> Result<TouchEvent> {
        self.interrupt_pin.wait_for_rising_edge().await?;
        self.read_last().await
    }
    
    // async fn detect(&mut self) -> Result<()> {

    // }

    // pub async fn init(&mut self) -> Result<()>
}
// void chsc6x_init(void)
// {
//     int i = 0;
//     int ret = 0;
//     unsigned char fw_update_ret_flag = 0; //1:update OK, !0 fail
//     struct ts_fw_infos fw_infos;
//     
//     for(i = 0; i < 3; i++) {
//         ret = chsc6x_tp_dect(&fw_infos, &fw_update_ret_flag);
//         if(1 == ret) {
//           #if CHSC6X_AUTO_UPGRADE /* If need update FW */
//             chsc6x_info("chsc6x_tp_dect succeed!\r\n");    
//             if(1 == fw_update_ret_flag) {
//                 chsc6x_err("update fw succeed! \r\n"); 
//                 break;
//             } else {
//                 chsc6x_err("update fw failed! \r\n"); 
//             }
//           #else
//             break;
//           #endif
//         }else {
//             chsc6x_err("chsc6x_tp_dect failed! i = %d \r\n", i);    
//         }
//     }
// }
// 
