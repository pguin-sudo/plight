use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use rand::random;
use serialport::{self, SerialPort};

use crate::config::CONFIG;
use crate::core::led_sequence::LedSequence;
use crate::core::strip::Strip;
use crate::errors::PLightError::{PostfixReading, WrongLength, WrongPostfix};
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

        // TODO: Try to replace unwrap() with ?
        let mut port = self.port.lock().unwrap();

        port.write_all(&PREFIX)?;

        let hi: u8 = random();
        let lo: u8 = random();
        let chk = (hi ^ lo ^ 0x55) as u8;

        port.write_all(&[hi])?;
        port.write_all(&[lo])?;
        port.write_all(&[chk])?;
        for led_color in led_colors {
            port.write_all(&led_color.apply_tint())?;
        }

        let mut buf = [0; 3];
        match port.read_exact(&mut buf) {
            Ok(_) => {
                buf.reverse();
                if buf == PREFIX {
                    Ok(())
                } else {
                    Err(WrongPostfix(buf).into())
                }
            }
            Err(e) => Err(PostfixReading(e).into()),
        }
    }
}
