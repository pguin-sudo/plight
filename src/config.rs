use confique::{Config, File, FileFormat, Partial};

const DEFAULT_CONFIG_PATH: &str = "/etc/plight/config.toml";

#[derive(Config)]
pub struct Conf {
    // Strip
    pub split_corners: bool,
    pub width: u8,
    pub hight: u8,
    pub bottom_gap: u8,
    pub clockwise: bool,

    // Colors
    pub order: String,

    // Ambilight
    pub accuracy: u8,
    pub thickness: u16,

    // General
    pub debug: bool,
}

impl Conf {
    pub fn new() -> Conf {
        type PartialConf = <Conf as Config>::Partial;
        let from_file: PartialConf = File::with_format(DEFAULT_CONFIG_PATH, FileFormat::Toml)
            .required()
            .load()
            .expect("Config loading (P-ERROR #0)");

        let manual = PartialConf {
            split_corners: Some(true),
            width: Some(29),
            hight: Some(15),
            bottom_gap: Some(7),
            clockwise: Some(false),
            order: Some("RGB".to_string()),
            accuracy: Some(5),
            thickness: Some(250),
            debug: Some(false),
        };

        let defaults = PartialConf::default_values();

        let merged = from_file.with_fallback(manual).with_fallback(defaults);
        Conf::from_partial(merged).expect("Config combination (P-ERROR #1)")
    }

    pub fn num_leds(&self) -> u8 {
        self.width * 2 + self.hight * 2 - self.bottom_gap
    }
}
