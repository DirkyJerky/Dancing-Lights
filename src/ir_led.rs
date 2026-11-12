use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output, Pin, Pull},
};

pub struct IRLed<'a> {
    vcc: Output<'a>,
    float: Input<'a>,
}

impl<'a> IRLed<'a> {
    pub fn new(vcc: Peri<'a, impl Pin>, float: Peri<'a, impl Pin>) -> Self {
        Self {
            vcc: Output::new(vcc, Level::Low),
            float: Input::new(float, Pull::None),
        }
    }

    pub fn up(&mut self) {
        self.vcc.set_high();
    }

    pub fn down(&mut self) {
        self.vcc.set_low();
    }

    pub fn reset(&mut self) {
        self.down();
    }
}
