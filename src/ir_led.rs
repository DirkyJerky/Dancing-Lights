use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
    peripherals::PWM_SLICE2,
    pwm::ChannelBPin,
};

use crate::utils::pwm_led::PWMLed;

/// The IR Led, as configured via a PWM handler
pub struct IRLed<'a> {
    /// The VCC pin
    pwm_led: PWMLed<'a, PWM_SLICE2>,
    /// A high impedence pin, attached to the LED cathode and to GND through a resistor.
    _float: Input<'a>,
}

impl<'a> IRLed<'a> {
    /// Construct the IRLed using the PWM slice, and vcc + float pins
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
