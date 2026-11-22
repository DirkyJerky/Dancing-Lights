use embassy_rp::{gpio::Pin, pwm::Slice};

use super::int_pid::{Limits, N};
use crate::led_array::{ColorPin, GroundPin, LEDArray};

// Compute a normalized 0..=65536 level using only 32-bit arithmetic.
fn normalized_level(offset: u32, span: u32) -> u32 {
    if offset == span {
        return 65_536;
    }

    let mut remainder = offset;
    let mut fraction = 0;
    for _ in 0..16 {
        fraction <<= 1;
        // Equivalent to doubling the remainder, comparing with span, and
        // subtracting span when needed, without overflowing u32.
        if remainder >= span - remainder {
            remainder -= span - remainder;
            fraction |= 1;
        } else {
            remainder *= 2;
        }
    }
    fraction
}

/// Displays a value within configured limits on four LED color channels.
pub struct ArrayGuage<
    'b,
    'peri,
    RS: Slice,
    RP: Pin,
    YS: Slice,
    YP: Pin,
    GS: Slice,
    GP: Pin,
    BS: Slice,
    BP: Pin,
> {
    limits: Limits,
    array: &'b mut LEDArray<'peri, RS, RP, YS, YP, GS, GP, BS, BP>,
}

impl<'b, 'peri, RS: Slice, RP: Pin, YS: Slice, YP: Pin, GS: Slice, GP: Pin, BS: Slice, BP: Pin>
    ArrayGuage<'b, 'peri, RS, RP, YS, YP, GS, GP, BS, BP>
{
    pub fn new(
        limits: Limits,
        array: &'b mut LEDArray<'peri, RS, RP, YS, YP, GS, GP, BS, BP>,
    ) -> Self {
        array.set_ground(GroundPin::Left, true);
        array.set_ground(GroundPin::Right, true);
        Self { limits, array }
    }

    /// Progressively fill the channels, reaching full brightness at 20%,
    /// 40%, 60%, and 80% of the range. Values outside the limits are clamped.
    pub fn set(&mut self, val: N) {
        let low = self.limits.low().to_bits();
        let high = self.limits.high().to_bits();
        let value = val.to_bits().clamp(low, high);

        // Unsigned differences cover even limits spanning the entire signed
        // fixed-point range without overflowing.
        let span = (high as u32).wrapping_sub(low as u32);
        let offset = (value as u32).wrapping_sub(low as u32);
        let level = normalized_level(offset, span);

        for (index, color) in [
            ColorPin::Red,
            ColorPin::Yellow,
            ColorPin::Green,
            ColorPin::Blue,
        ]
        .into_iter()
        .enumerate()
        {
            let brightness = (level * 5)
                .saturating_sub(index as u32 * 65_536)
                .min(u16::MAX as u32) as u16;
            self.array.set_color_brightness(color, brightness);
        }
    }
}
