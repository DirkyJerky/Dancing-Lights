use embassy_rp::{
    Peri,
    gpio::{Flex, Pin, Pull},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinState {
    High,
    Low,
    HighImpedance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrayPin {
    Red,
    Yellow,
    Green,
    Blue,
    Bottom,
    Top,
}

fn set_state(pin: &mut Flex<'_>, state: PinState) {
    match state {
        PinState::High => {
            // Prepare the latch before enabling output to avoid driving a stale level.
            pin.set_high();
            pin.set_as_output();
        }
        PinState::Low => {
            pin.set_low();
            pin.set_as_output();
        }
        PinState::HighImpedance => {
            pin.set_as_input();
            pin.set_pull(Pull::None);
        }
    }
}

fn floating_pin<'a>(pin: Peri<'a, impl Pin>) -> Flex<'a> {
    let mut pin = Flex::new(pin);
    set_state(&mut pin, PinState::HighImpedance);
    pin
}

pub struct LEDArray<'a> {
    red: Flex<'a>,
    yellow: Flex<'a>,
    green: Flex<'a>,
    blue: Flex<'a>,
    bottom_gnd: Flex<'a>,
    top_gnd: Flex<'a>,
}

impl<'a> LEDArray<'a> {
    pub fn new(
        red: Peri<'a, impl Pin>,
        yellow: Peri<'a, impl Pin>,
        green: Peri<'a, impl Pin>,
        blue: Peri<'a, impl Pin>,
        bottom_gnd: Peri<'a, impl Pin>,
        top_gnd: Peri<'a, impl Pin>,
    ) -> Self {
        Self {
            red: floating_pin(red),
            yellow: floating_pin(yellow),
            green: floating_pin(green),
            blue: floating_pin(blue),
            bottom_gnd: floating_pin(bottom_gnd),
            top_gnd: floating_pin(top_gnd),
        }
    }

    pub fn set_pin(&mut self, pin: ArrayPin, state: PinState) {
        let pin = match pin {
            ArrayPin::Red => &mut self.red,
            ArrayPin::Yellow => &mut self.yellow,
            ArrayPin::Green => &mut self.green,
            ArrayPin::Blue => &mut self.blue,
            ArrayPin::Bottom => &mut self.bottom_gnd,
            ArrayPin::Top => &mut self.top_gnd,
        };
        set_state(pin, state);
    }

    pub fn blank(&mut self) {
        for pin in [
            &mut self.red,
            &mut self.yellow,
            &mut self.green,
            &mut self.blue,
            &mut self.bottom_gnd,
            &mut self.top_gnd,
        ] {
            set_state(pin, PinState::HighImpedance);
        }
    }
}
