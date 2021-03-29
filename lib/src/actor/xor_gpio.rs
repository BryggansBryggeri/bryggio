use super::simple_gpio::SimpleGpioActor;
#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

pub struct XorActor {
    gpio_one: SimpleGpioActor<hardware_impl::GpioPin>,
    gpio_two: SimpleGpioActor<hardware_impl::GpioPin>,
}
