use embassy_rp::{
    Peri,
    gpio::{Flex, Pin, Pull},
};

/// Possible states of the pins in the LED Array
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinState {
    High,
    Low,
    HighImpedance,
}

/// Pins in the array that can be referenced
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrayPin {
    /// Red row
    Red,
    /// Yellow row
    Yellow,
    /// Green row
    Green,
    /// Blue row
    Blue,
    /// Bottom column (towards bottom of breadboard)
    Bottom,
    /// Top column (towards top of breadboard)
    Top,
}

/// Set the state of an individual pin
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

/// Setup a pin as a floating pin, that can switch between input and output modes
fn floating_pin<'a>(pin: Peri<'a, impl Pin>) -> Flex<'a> {
    let mut pin = Flex::new(pin);
    set_state(&mut pin, PinState::HighImpedance);
    pin
}

/// A 2x4 addressable LED array
///
/// Each LED array consists of 4 rows, each row consisting of two LEDs of a single color.
/// In order, the color rows are:
/// - Red
/// - Yellow
/// - Green
/// - Blue
///
/// Each row of colors and also the top/bottom columns are individually addressable
///
pub struct LEDArray<'a> {
    red: Flex<'a>,
    yellow: Flex<'a>,
    green: Flex<'a>,
    blue: Flex<'a>,
    bottom_gnd: Flex<'a>,
    top_gnd: Flex<'a>,
}

impl<'a> LEDArray<'a> {
    /// Create a new LED array from the referenced pins
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

    /// Set a specific pin to the provided state
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

    /// Reset all pins to HighImpedance
    pub fn reset(&mut self) {
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
