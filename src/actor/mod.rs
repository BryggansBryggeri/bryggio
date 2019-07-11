pub mod dummy;
pub mod simple_gpio;
pub mod xor;

pub trait Actor {
    fn set_signal(&self, signal: f32) -> Result<(), gpio_cdev::errors::Error>;
}
