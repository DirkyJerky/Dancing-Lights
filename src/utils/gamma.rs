//! Gamma correction
//!
//! Maps a 16-bit brightness value through a gamma 2.2 curve using a
//! 256-entry lookup table generated at compile time.

const LUT_SIZE: usize = 256;
const Q16_MAX: u32 = u16::MAX as u32;

const fn mul_q16(a: u32, b: u32) -> u32 {
    (a * b) / Q16_MAX
}

const fn pow5_q16(x: u32) -> u32 {
    let x2 = mul_q16(x, x);
    let x4 = mul_q16(x2, x2);
    mul_q16(x4, x)
}

const fn fifth_root_q16(x: u32) -> u32 {
    let mut low = 0;
    let mut high = Q16_MAX;

    while low < high {
        let mid = low + (high - low + 1) / 2;
        if pow5_q16(mid) <= x {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    low
}

const fn gamma22_value(x: u32) -> u16 {
    // x^2.2 = x^2 * fifth_root(x)
    let x2 = mul_q16(x, x);
    let root = fifth_root_q16(x);
    mul_q16(x2, root) as u16
}

const fn make_gamma22_lut() -> [u16; LUT_SIZE] {
    let mut lut = [0; LUT_SIZE];
    let mut i = 0;

    while i < LUT_SIZE {
        let x = (i as u32 * Q16_MAX) / (LUT_SIZE as u32 - 1);
        lut[i] = gamma22_value(x);
        i += 1;
    }

    lut
}

static GAMMA22_LUT: [u16; LUT_SIZE] = make_gamma22_lut();

/// Apply a gamma 2.2 curve to a 16-bit brightness value.
///
/// The lookup-table entries are linearly interpolated so the input and output
/// both retain 16-bit resolution.
pub fn gamma22(x: u16) -> u16 {
    let scaled = x as u32 * (LUT_SIZE as u32 - 1);
    let index = (scaled / Q16_MAX) as usize;
    let remainder = scaled % Q16_MAX;

    if index >= LUT_SIZE - 1 {
        return GAMMA22_LUT[LUT_SIZE - 1];
    }

    let start = GAMMA22_LUT[index] as u32;
    let end = GAMMA22_LUT[index + 1] as u32;
    let difference = end - start;

    (start + difference * remainder / Q16_MAX) as u16
}
