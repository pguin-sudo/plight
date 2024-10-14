use rand::random;
use rgb::RGB8;
use serialport::{self, SerialPort};

use crate::config::{StripConf, TintConf};

const PREFIX: [u8; 3] = [89, 124, 234];

pub struct Strip {
    port: Box<dyn SerialPort>,
    tint_conf: TintConf,
    strip_length: usize,
}

impl Strip {
    pub fn new(conf: &StripConf) -> Strip {
        Strip {
            port: serialport::new(conf.serial_port.clone(), conf.baudrate)
                .open()
                .expect("Failed to open port"),
            strip_length: conf.length() as usize,
            tint_conf: conf.tint.clone(),
        }
    }

    pub fn set_leds(&mut self, led_colors: Vec<RGB8>) {
        if led_colors.len() != self.strip_length {
            println!(
                "Wrong strip length to set ({} is not {})",
                led_colors.len(),
                self.strip_length
            );
            return;
        }

        let _ = self.port.write(&PREFIX);

        let hi: u8 = random();
        let lo: u8 = random();
        let chk = (hi ^ lo ^ 0x55) as u8;

        let _ = self.port.write(&[hi]);
        let _ = self.port.write(&[lo]);
        let _ = self.port.write(&[chk]);

        for rgba in led_colors {
            let (r, g, b) = match self.tint_conf.order.as_str() {
                "RGB" => (rgba.r, rgba.g, rgba.b),
                "GRB" => (rgba.g, rgba.r, rgba.b),
                "BRG" => (rgba.b, rgba.r, rgba.g),
                "BGR" => (rgba.b, rgba.g, rgba.r),
                "RBG" => (rgba.r, rgba.b, rgba.g),
                _ => (rgba.r, rgba.g, rgba.b),
            };

            let _ = self.port.write(&[r, g, b]);
        }
    }
}
