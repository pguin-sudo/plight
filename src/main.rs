mod config;
mod strip;

use config::Conf;
use rand::Rng;
use strip::Strip;

fn main() {
    let config = Conf::new();
    let mut strip = Strip::new("/dev/ttyUSB0");

    let mut rng = rand::thread_rng();

    loop {
        let led_colors: Vec<(u8, u8, u8)> = (0..config.num_leds())
            .map(|_| {
                let r = rng.gen_range(0..=255);
                let g = rng.gen_range(0..=255);
                let b = rng.gen_range(0..=255);
                (r, g, b)
            })
            .collect();

        strip.set_leds(led_colors)
    }
}
