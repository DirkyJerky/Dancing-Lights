use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
    peripherals::{PIN_12, PIN_14, PIN_15, PWM_SLICE6, PWM_SLICE7},
};

use crate::utils::pwm_led::PWMLed;

pub struct RGBLed<'a> {
    pwm_red: PWMLed<'a, PWM_SLICE6>,
    _float: Input<'a>,
    pwm_blue: PWMLed<'a, PWM_SLICE7>,
    pwm_green: PWMLed<'a, PWM_SLICE7>,
}

impl<'a> RGBLed<'a> {
    pub fn new(
        red: Peri<'a, PIN_12>,
        float: Peri<'a, impl Pin>,
        blue: Peri<'a, PIN_14>,
        green: Peri<'a, PIN_15>,
        slice_red: Peri<'a, PWM_SLICE6>,
        slice_green_blue: Peri<'a, PWM_SLICE7>,
    ) -> Self {
        let (pwm_blue, pwm_green) = PWMLed::new_ab(slice_green_blue, blue, green);
        Self {
            pwm_red: PWMLed::new_a(slice_red, red),
            _float: Input::new(float, Pull::None),
            pwm_blue,
            pwm_green,
        }
    }

    pub fn set(&mut self, r: u16, g: u16, b: u16) {
        self.pwm_red.set_brightness(r);
        self.pwm_green.set_brightness(g);
        self.pwm_blue.set_brightness(b);
    }

    pub fn up(&mut self) {
        self.set(u16::MAX, u16::MAX, u16::MAX)
    }

    pub fn reset(&mut self) {
        self.set(0, 0, 0)
    }

    pub fn set_hsl(&mut self) {}
}
