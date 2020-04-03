use crate::hardware::dummy::{Delay, GpioPin};
use crate::sensor;
use onewire as ext_dsb;

pub struct Ds18b20 {
    pub id: String,
    ds18: ext_dsb::DS18B20,
}

impl Ds18b20 {
    pub fn new(id: &str) -> Ds18b20 {
        let mut gpio = GpioPin::new(4, "dsb");
        let mut delay = Delay {};
        let mut wire = ext_dsb::OneWire::new(&mut gpio, false);
        if wire.reset(&mut delay).is_err() {
            panic!("Missing pullup or error on line");
        };
        let device = ext_dsb::Device::from_str("Dummy address").expect("This will panic");
        let ds18b20 = ext_dsb::DS18B20::new::<sensor::Error>(device).unwrap();
        Ds18b20 {
            id: id.into(),
            ds18: ds18b20,
        }
    }
}
