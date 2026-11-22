use core::marker::PhantomData;

use embassy_rp::{
    Peri,
    gpio::Pin,
    pwm::{ChannelAPin, ChannelBPin, Config, Pwm, PwmOutput, SetDutyCycle, Slice},
};

pub struct PWMLed<'a, S: Slice, P: Pin> {
    pwm: PwmOutput<'a>,
    _marker_slice: PhantomData<S>,
    _marker_pin: PhantomData<P>,
}

impl<'a, S: Slice, P: Pin> PWMLed<'a, S, P> {
    fn config() -> Config {
        // PWM configuration is copied from
        // https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/pwm.rs
        let desired_freq_hz = 10_000;
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 16u8;
        let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;

        let mut config = Config::default();
        config.top = period;
        config.divider = divider.into();

        config
    }

    pub fn new_b(slice: Peri<'a, S>, vcc: Peri<'a, P>) -> Self
    where
        P: ChannelBPin<S>,
    {
        Self {
            pwm: Pwm::new_output_b(slice, vcc, Self::config())
                .split()
                .1
                .unwrap(),
            _marker_slice: PhantomData,
            _marker_pin: PhantomData,
        }
    }

    pub fn new_a(slice: Peri<'a, S>, vcc: Peri<'a, P>) -> Self
    where
        P: ChannelAPin<S>,
    {
        Self {
            pwm: Pwm::new_output_a(slice, vcc, Self::config())
                .split()
                .0
                .unwrap(),
            _marker_slice: PhantomData,
            _marker_pin: PhantomData,
        }
    }

    pub fn new_ab<PB>(
        slice: Peri<'a, S>,
        vcc_a: Peri<'a, P>,
        vcc_b: Peri<'a, PB>,
    ) -> (Self, PWMLed<'a, S, PB>)
    where
        P: ChannelAPin<S>,
        PB: Pin + ChannelBPin<S>,
    {
        let pwm = Pwm::new_output_ab(slice, vcc_a, vcc_b, Self::config());
        let (a, b) = pwm.split();
        (
            Self {
                pwm: a.unwrap(),
                _marker_slice: PhantomData,
                _marker_pin: PhantomData,
            },
            PWMLed {
                pwm: b.unwrap(),
                _marker_slice: PhantomData,
                _marker_pin: PhantomData,
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
