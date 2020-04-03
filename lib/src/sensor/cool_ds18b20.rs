/// DS18B20 temperature sensor
///
/// [Datasheet](https://datasheets.maximintegrated.com/en/ds/DS18B20.pdf)
///
/// Wraps the [onewire](https://crates.io/crates/onewire) implementation
use crate::sensor;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use onewire as ext_ds18;

pub struct Ds18b20<G, D>
where
    G: ext_ds18::OpenDrainOutput<sensor::Error> + Send,
    D: DelayMs<u16> + DelayUs<u16> + Send,
{
    pub id: String,
    ds18: ext_ds18::DS18B20,
    gpio: G,
    delay: D,
}

impl<G, D> Ds18b20<G, D>
where
    G: ext_ds18::OpenDrainOutput<sensor::Error> + Send,
    D: DelayMs<u16> + DelayUs<u16> + Send,
{
    pub fn new(id: &str, gpio: G, delay: D) -> Ds18b20<G, D> {
        let device = ext_ds18::Device::from_str("Dummy address").expect("This will panic");
        let ds18b20 = ext_ds18::DS18B20::new::<sensor::Error>(device).unwrap();
        Ds18b20 {
            id: id.into(),
            ds18: ds18b20,
            gpio,
            delay,
        }
    }
}

impl<G, D> sensor::Sensor for Ds18b20<G, D>
where
    G: ext_ds18::OpenDrainOutput<sensor::Error> + Send,
    D: DelayMs<u16> + DelayUs<u16> + Send,
{
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_measurement(&mut self) -> Result<f32, sensor::Error> {
        // TODO: Not nice to recreate wire every time?
        // but have not found any way to add a ref to it to self.
        let mut wire = ext_ds18::OneWire::new(&mut self.gpio, false);
        if wire.reset(&mut self.delay).is_err() {
            panic!("Missing pullup or error on line");
        };
        // request sensor to measure temperature
        let resolution = self
            .ds18
            .measure_temperature(&mut wire, &mut self.delay)
            .unwrap();
        // wait for compeltion, depends on resolution
        self.delay.delay_ms(resolution.time_ms());
        // read temperature
        match self.ds18.read_temperature(&mut wire, &mut self.delay) {
            Ok(temp) => {
                let (integer, decimal) = ext_ds18::ds18b20::split_temp(temp);
                Ok(int_tuple_to_float(integer, decimal))
            }
            Err(_err) => todo!("Better error for underlying error",),
        }
    }
}

fn int_tuple_to_float(integer: i16, decimal: i16) -> f32 {
    integer as f32 + decimal as f32 / 10_000.0
}
