use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
    peripherals::PWM_SLICE2,
    pwm::ChannelBPin,
};

use crate::utils::pwm_led::PWMLed;

pub struct IRLed<'a> {
    pwm_led: PWMLed<'a, PWM_SLICE2>,
    _float: Input<'a>,
}

impl<'a> IRLed<'a> {
    pub fn new(
        slice: Peri<'a, PWM_SLICE2>,
        vcc: Peri<'a, impl ChannelBPin<PWM_SLICE2>>,
        float: Peri<'a, impl Pin>,
    ) -> Self {
        Self {
            pwm_led: PWMLed::new_b(slice, vcc),
            _float: Input::new(float, Pull::None),
        }
    }

    pub fn up(&mut self) {
        self.set_duty_cycle(u16::MAX);
    }

    pub fn down(&mut self) {
        self.set_duty_cycle(0);
    }

    pub fn set_duty_cycle(&mut self, duty_cycle: u16) {
        self.pwm_led.set_duty_cycle(duty_cycle)
    }

    pub fn set_brightness(&mut self, brightness: u16) {
        self.pwm_led.set_brightness(brightness);
    }

    pub fn reset(&mut self) {
        self.down();
    }
}
