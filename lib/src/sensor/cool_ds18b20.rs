use crate::hardware::dummy::{Delay, GpioPin};
use onewire as ext_dsb;

#[derive(Debug)]
pub struct DSB1820 {
    pub id: String,
    dsb: ext_dsb::DSB1820,
}

impl DSB1820 {
    pub fn new(id: &str) -> DSB1820 {
        let gpio = GpioPin::new(4, "dsb");
        let delay = Delay {};
        let mut wire = ext_dsb::OneWire::new(&mut gpio, false);
        if wire.reset(&mut delay).is_err() {
            panic!("Missing pullup or error on line");
        };
        let device = ext_dsb::Device::from_str("Dummy address").expect("This will panic");
        let mut ds18b20 = ext_dsb::DS18b20::new(device).unwrap();
    }
}
