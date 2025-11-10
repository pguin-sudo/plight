use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use log::{error, warn};
use rand::random;
use serialport::{self, SerialPort};

use crate::config::CONFIG;
use crate::core::led_sequence::LedSequence;
use crate::core::strip::Strip;
use crate::errors::PLightError::{PostfixReading, WrongLength};

const PREFIX: [u8; 3] = [89, 124, 234];

#[derive(Clone)]
pub struct ArduinoStrip {
    port: Arc<Mutex<Box<dyn SerialPort + Send>>>,
    strip_length: usize,
}

impl Strip for ArduinoStrip {
    fn new() -> Result<Self>
    where
        Self: Sized,
    {
        let port: Box<dyn SerialPort + Send> =
            serialport::new(CONFIG.strip.serial_port.clone(), CONFIG.strip.baudrate)
                .timeout(Duration::from_millis(1000))
                .open()?;

        Ok(ArduinoStrip {
            port: Arc::new(Mutex::new(port)),
            strip_length: CONFIG.strip.len(),
        })
    }

    fn set_leds(&self, led_colors: &LedSequence) -> Result<()> {
        if led_colors.len() != self.strip_length {
            return Err(WrongLength {
                given: led_colors.len(),
                actual: self.strip_length,
            }
            .into());
        }

        let mut port = match self.port.lock() {
            Ok(port) => port,
            Err(e) => {
                error!("Serial port error: {}", e);
                return Ok(());
            }
        };

        let _ = port.clear(serialport::ClearBuffer::Input);

        port.write_all(&PREFIX)?;

        let hi: u8 = random();
        let lo: u8 = random();
        let chk = hi ^ lo ^ 0x55;

        port.write_all(&[hi, lo, chk])?;

        for led_color in led_colors {
            let color_bytes = led_color.apply_tint();
            port.write_all(&color_bytes)?;
        }

        port.flush()?;

        let mut buf = [0; 3];
        match port.read_exact(&mut buf) {
            Ok(_) => {
                buf.reverse();
                if buf != PREFIX {
                    warn!("Wrong postfix {:?}, expected {:?}", buf, PREFIX);
                }
                Ok(())
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    warn!("Timeout reading postfix - data may have been sent successfully");
                    Ok(())
                } else {
                    Err(PostfixReading(e).into())
                }
            }
        }
    }
}
