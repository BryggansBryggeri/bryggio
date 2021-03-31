use super::{simple_gpio::SimpleGpioActor, ActorError, ActorSignal};
#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;
use crate::{actor::Actor, hardware::dummy::GpioState};

pub struct XorActor {
    gpio_one: SimpleGpioActor<hardware_impl::GpioPin>,
    gpio_two: SimpleGpioActor<hardware_impl::GpioPin>,
}

impl XorActor {
    fn contains_id(&self, signal_id: &str) -> bool {
        self.gpio_one.id == signal_id || self.gpio_two.id == signal_id
    }
    fn xor(&self, high_id: &str) -> bool {
        let one = high_id == self.gpio_one.id && self.gpio_two.state() == GpioState::High;
        let two = high_id == self.gpio_two.id && self.gpio_two.state() == GpioState::High;
        one || two
    }
}

impl Actor for XorActor {
    fn set_signal(&mut self, signal: &ActorSignal) -> Result<(), ActorError> {
        self.validate_signal(signal)?;
        if signal.id == self.gpio_one.id {
            self.gpio_one.set_signal(signal)
        } else if signal.id == self.gpio_two.id {
            self.gpio_two.set_signal(signal)
        } else {
            unreachable!()
        }
    }
    fn validate_signal(&self, signal: &ActorSignal) -> Result<(), ActorError> {
        if self.contains_id(&signal.id) {
            return Err(ActorError::Generic(String::from("id")));
        };

        if signal.signal == 0.0 {
            return Ok(());
        };

        if self.xor(&signal.id) {
            Err(ActorError::Generic(String::from("xor")))
        } else {
            Ok(())
        }
    }
}
