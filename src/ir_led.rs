use core::marker::PhantomData;

use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
    pwm::{ChannelBPin, Config, Pwm, SetDutyCycle, Slice},
};

pub struct IRLed<'a, S: Slice> {
    pwm: Pwm<'a>,
    _float: Input<'a>,
    _marker: PhantomData<S>,
}

impl<'a, S: Slice> IRLed<'a, S> {
    pub fn new(
        slice: Peri<'a, S>,
        vcc: Peri<'a, impl ChannelBPin<S>>,
        float: Peri<'a, impl Pin>,
    ) -> Self {
        // PWM configuration is copied from
        // https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/pwm.rs
        let desired_freq_hz = 10_000;
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 16u8;
        let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;

        let mut config = Config::default();
        config.top = period;
        config.divider = divider.into();

        Self {
            pwm: Pwm::new_output_b(slice, vcc, config),
            _float: Input::new(float, Pull::None),
            _marker: PhantomData,
        }
    }

    pub fn up(&mut self) {
        if let Err(err) = self.pwm.set_duty_cycle_fully_on() {}
    }

    pub fn down(&mut self) {
        if let Err(err) = self.pwm.set_duty_cycle_fully_off() {}
    }

    pub fn set_duty_cycle(&mut self, duty_cycle: u16) {
        if let Err(err) = self.pwm.set_duty_cycle_fraction(duty_cycle, u16::MAX) {}
    }

    pub fn reset(&mut self) {
        self.down();
    }
}
