use serialport::{self, SerialPort};

const PREFIX: &[u8] = &[89, 124, 234];

pub struct Strip {
    port: Box<dyn SerialPort>,
}

impl Strip {
    pub fn new(port_name: &str) -> Strip {
        Strip {
            port: serialport::new(port_name, 115200)
                .open()
                .expect("Failed to open port"),
        }
    }

    pub fn set_leds(&mut self, led_colors: Vec<(u8, u8, u8)>) {
        let _ = self.port.write_all(&PREFIX);

        let hi: u8 = 0x13;
        let lo: u8 = 0x11;
        let chk = (hi ^ lo ^ 0x55) as u8;

        let _ = self.port.write(&[hi]);
        let _ = self.port.write(&[lo]);
        let _ = self.port.write(&[chk]);

        for (r, g, b) in led_colors {
            let _ = self.port.write(&[r]);
            let _ = self.port.write(&[g]);
            let _ = self.port.write(&[b]);
        }
    }
}
