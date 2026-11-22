use embassy_rp::peripherals::{PIN_13, PIN_14, PIN_15, PWM_SLICE6, PWM_SLICE7};

use crate::utils::pwm_led::PWMLed;

/// An RGB Led, with each R/G/B component configured via [`PWMLed`]
/// for gamma-corrected brightness control
pub struct RGBLed<'a> {
    pwm_red: PWMLed<'a, PWM_SLICE6, PIN_13>,
    pwm_blue: PWMLed<'a, PWM_SLICE7, PIN_14>,
    pwm_green: PWMLed<'a, PWM_SLICE7, PIN_15>,
}

impl<'a> RGBLed<'a> {
    /// Construct a ne RGBLed given the provided peripherals and PWM control slices
    pub fn new(
        red: PWMLed<'static, PWM_SLICE6, PIN_13>,
        blue: PWMLed<'static, PWM_SLICE7, PIN_14>,
        green: PWMLed<'static, PWM_SLICE7, PIN_15>,
    ) -> Self {
        Self {
            pwm_red: red,
            pwm_blue: blue,
            pwm_green: green,
        }
    }

    /// The brightness of each color to the provided values
    pub fn set(&mut self, r: u16, g: u16, b: u16) {
        self.pwm_red.set_brightness(r);
        self.pwm_green.set_brightness(g);
        self.pwm_blue.set_brightness(b);
    }

    /// Set all color components to the maximum value
    pub fn up(&mut self) {
        self.set(u16::MAX, u16::MAX, u16::MAX)
    }

    /// Sets all color components to off
    pub fn reset(&mut self) {
        self.set(0, 0, 0)
    }

    /// Set the color from HSL components, each scaled from 0 to `u16::MAX`.
    /// Hue wraps around, so 0 and `u16::MAX` both represent red.
    pub fn set_hsl(&mut self, h: u16, s: u16, l: u16) {
        const MAX: u32 = u16::MAX as u32;

        let hue = h as u32 * 6;
        let sector = (hue / MAX) % 6;
        let hue_fraction = hue % MAX;

        let twice_lightness = 2 * l as u32;
        let lightness_distance = twice_lightness.abs_diff(MAX);
        let chroma = (lightness_distance * s as u32) / MAX;
        let x = if sector % 2 == 0 {
            chroma * hue_fraction / MAX
        } else {
            chroma * (MAX - hue_fraction) / MAX
        };
        let match_value = l as u32 - chroma / 2;

        let (r, g, b) = match sector {
            0 => (chroma, x, 0),
            1 => (x, chroma, 0),
            2 => (0, chroma, x),
            3 => (0, x, chroma),
            4 => (x, 0, chroma),
            _ => (chroma, 0, x),
        };

        self.set(
            (r + match_value) as u16,
            (g + match_value) as u16,
            (b + match_value) as u16,
        );
    }
}
