//! Provides color brightness control and column selection for a 2×4 LED array.

use embassy_rp::{
    Peri,
    gpio::{Flex, Pin, Pull},
    pwm::Slice,
};

use crate::utils::pwm_led::PWMLed;

/// Color pin rows in the array that can be referenced
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorPin {
    /// Red row
    Red,
    /// Yellow row
    Yellow,
    /// Green row
    Green,
    /// Blue row
    Blue,
}

/// Ground pins columns that can be enabled or disabled
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundPin {
    /// Left column
    Left,
    /// Right column
    Right,
}

/// Setup a pin as a floating pin, that can switch between input and output modes
fn floating_pin<'a>(pin: Peri<'a, impl Pin>) -> Flex<'a> {
    let mut pin = Flex::new(pin);
    pin.set_as_input();
    pin.set_pull(Pull::None);

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
pub struct LEDArray<
    'a,
    RS: Slice,
    RP: Pin,
    YS: Slice,
    YP: Pin,
    GS: Slice,
    GP: Pin,
    BS: Slice,
    BP: Pin,
> {
    red: PWMLed<'a, RS, RP>,
    yellow: PWMLed<'a, YS, YP>,
    green: PWMLed<'a, GS, GP>,
    blue: PWMLed<'a, BS, BP>,
    right_gnd: Flex<'a>,
    left_gnd: Flex<'a>,
}

impl<'a, RS: Slice, RP: Pin, YS: Slice, YP: Pin, GS: Slice, GP: Pin, BS: Slice, BP: Pin>
    LEDArray<'a, RS, RP, YS, YP, GS, GP, BS, BP>
{
    /// Create a new LED array from the referenced pins
    pub fn new(
        red: PWMLed<'a, RS, RP>,
        yellow: PWMLed<'a, YS, YP>,
        green: PWMLed<'a, GS, GP>,
        blue: PWMLed<'a, BS, BP>,
        right_gnd: Peri<'a, impl Pin>,
        left_gnd: Peri<'a, impl Pin>,
    ) -> Self {
        Self {
            red,
            yellow,
            green,
            blue,
            right_gnd: floating_pin(right_gnd),
            left_gnd: floating_pin(left_gnd),
        }
    }

    pub fn set_color_brightness(&mut self, color: ColorPin, brightness: u16) {
        match color {
            ColorPin::Red => self.red.set_brightness(brightness),
            ColorPin::Yellow => self.yellow.set_brightness(brightness),
            ColorPin::Green => self.green.set_brightness(brightness),
            ColorPin::Blue => self.blue.set_brightness(brightness),
        }
    }

    pub fn set_ground(&mut self, column: GroundPin, up: bool) {
        let pin = match column {
            GroundPin::Left => &mut self.left_gnd,
            GroundPin::Right => &mut self.right_gnd,
        };

        if up {
            pin.set_low();
            pin.set_as_output();
        } else {
            pin.set_as_input();
            pin.set_pull(Pull::None);
        }
    }

    /// Reset all pins to HighImpedance
    pub fn reset(&mut self) {
        self.set_color_brightness(ColorPin::Red, 0);
        self.set_color_brightness(ColorPin::Yellow, 0);
        self.set_color_brightness(ColorPin::Green, 0);
        self.set_color_brightness(ColorPin::Blue, 0);

        self.set_ground(GroundPin::Left, false);
        self.set_ground(GroundPin::Right, false);
    }
}
