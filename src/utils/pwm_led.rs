use core::marker::PhantomData;

use embassy_rp::{
    Peri,
    pwm::{ChannelAPin, ChannelBPin, Config, Pwm, PwmOutput, SetDutyCycle, Slice},
};

pub struct PWMLed<'a, S: Slice> {
    pwm: PwmOutput<'a>,
    _marker: PhantomData<S>,
}

impl<'a, S: Slice> PWMLed<'a, S> {
    pub fn new_b(slice: Peri<'a, S>, vcc: Peri<'a, impl ChannelBPin<S>>) -> Self {
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
            pwm: Pwm::new_output_b(slice, vcc, config).split().1.unwrap(),
            _marker: PhantomData,
        }
    }

    pub fn new_a(slice: Peri<'a, S>, vcc: Peri<'a, impl ChannelAPin<S>>) -> Self {
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
            pwm: Pwm::new_output_a(slice, vcc, config).split().0.unwrap(),
            _marker: PhantomData,
        }
    }

    pub fn new_ab(
        slice: Peri<'a, S>,
        vcc_a: Peri<'a, impl ChannelAPin<S>>,
        vcc_b: Peri<'a, impl ChannelBPin<S>>,
    ) -> (Self, Self) {
        // PWM configuration is copied from
        // https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/pwm.rs
        let desired_freq_hz = 10_000;
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 16u8;
        let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;

        let mut config = Config::default();
        config.top = period;
        config.divider = divider.into();

        let pwm = Pwm::new_output_ab(slice, vcc_a, vcc_b, config);
        let (a, b) = pwm.split();
        (
            Self {
                pwm: a.unwrap(),
                _marker: PhantomData,
            },
            Self {
                pwm: b.unwrap(),
                _marker: PhantomData,
            },
        )
    }

    pub fn up(&mut self) {
        self.set_duty_cycle(u16::MAX);
    }

    pub fn reset(&mut self) {
        self.set_duty_cycle(0);
    }

    pub fn set_duty_cycle(&mut self, duty_cycle: u16) {
        if let Err(_err) = self.pwm.set_duty_cycle_fraction(duty_cycle, u16::MAX) {
            defmt::debug!("Error setting duty cycle");
        }
    }

    /// Set perceived brightness after applying gamma 2.2 correction.
    pub fn set_brightness(&mut self, brightness: u16) {
        let corrected_duty = crate::utils::gamma::gamma22(brightness);
        self.set_duty_cycle(corrected_duty);
    }
}
