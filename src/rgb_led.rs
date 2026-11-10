use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output, Pin, Pull},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorPin {
    Red,
    Green,
    Blue,
}

pub struct RGBLed<'a> {
    red: Output<'a>,
    float: Input<'a>,
    blue: Output<'a>,
    green: Output<'a>,
}

impl<'a> RGBLed<'a> {
    pub fn new(
        red: Peri<'a, impl Pin>,
        float: Peri<'a, impl Pin>,
        blue: Peri<'a, impl Pin>,
        green: Peri<'a, impl Pin>,
    ) -> Self {
        Self {
            red: Output::new(red, Level::Low),
            float: Input::new(float, Pull::None),
            blue: Output::new(blue, Level::Low),
            green: Output::new(green, Level::Low),
        }
    }

    pub fn set_pin(&mut self, pin: ColorPin, on: bool) {
        let output = match pin {
            ColorPin::Red => &mut self.red,
            ColorPin::Green => &mut self.green,
            ColorPin::Blue => &mut self.blue,
        };

        if on {
            output.set_high();
        } else {
            output.set_low();
        }
    }
}
