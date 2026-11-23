//! Provides on/off and variable brightness control for the standalone LED.

use embassy_rp::peripherals::{PIN_12, PWM_SLICE6};

use crate::utils::pwm_led::PWMLed;

/// The IR Led, as configured via a PWM handler
pub struct IRLed<'a> {
    /// The VCC pin
    pwm_led: PWMLed<'a, PWM_SLICE6, PIN_12>,
}

impl<'a> IRLed<'a> {
    /// Construct the IRLed using the PWM slice, and vcc + float pins
    pub fn new(vcc: PWMLed<'static, PWM_SLICE6, PIN_12>) -> Self {
        Self { pwm_led: vcc }
    }

    /// Set the LED to maximum brightness
    pub fn up(&mut self) {
        self.set_duty_cycle(u16::MAX);
    }

    /// Turn the LED off
    pub fn down(&mut self) {
        self.set_duty_cycle(0);
    }

    /// Set the duty cycle for the PWM controller.
    /// u16::MAX is fully on
    /// 0 is fully off
    pub fn set_duty_cycle(&mut self, duty_cycle: u16) {
        self.pwm_led.set_duty_cycle(duty_cycle)
    }

    /// Set the percieved brightness of the LED
    /// u16::MAX is maximum brightness
    /// 0 is off
    pub fn set_brightness(&mut self, brightness: u16) {
        self.pwm_led.set_brightness(brightness);
    }

    /// Turn the LED off
    pub fn reset(&mut self) {
        self.down();
    }
}
