//! Trigonometric estimations
//!
//! Calculates sin(x) and cos(x) in 32-bit integer space.
//!
//! Stretches the 0..2*pi domain to cover 0..u16::MAX instead
//! Likewise, the range is stretched from -1..1 to cover i16::MIN...i16::MAX
//! => "sin"(x) = i16::MAX * sin(x * 2pi / u16::MAX)
//!
//! This library provides a core numerical estimation for sin(x) over
//! the range 0..pi/2 using the first five non-zero terms of the
//! Taylor series expansion.
//!
//! By relying on the symmetrical properties of sinx and cosx, this
//! estimation can be used to implement the full range of values for
//! sinx and cosx

const LOOKUP_SIZE: usize = 256;

const SIN_LUT: [i32; LOOKUP_SIZE + 1] = {
    let mut table = [0i32; LOOKUP_SIZE + 1];
    let mut i = 0;
    while i <= LOOKUP_SIZE {
        // Compute taylor series approximation
        // sin(x) = x - x^3/6 + x^5/120 - ...

        let x = (i as f64 / LOOKUP_SIZE as f64) * core::f64::consts::FRAC_PI_2;
        let x2 = x * x;
        let sin_x =
            x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))));
        table[i] = (sin_x * (i32::MAX as f64) + 0.5) as i32;
        i += 1;
    }
    table
};

pub(crate) fn sin(x: u32) -> i32 {
    const QUARTER_PHASE: u32 = 1 << 30;
    const FRACTION_MASK: u32 = QUARTER_PHASE - 1;

    let quadrant = x >> 30;
    let phase = x & FRACTION_MASK;
    let reflected = quadrant == 1 || quadrant == 3;
    let negative = quadrant >= 2;
    let quarter_position = if reflected {
        QUARTER_PHASE - phase
    } else {
        phase
    };

    // Convert the position within this quadrant into a LUT index and a
    // fractional distance to the next entry.
    let lut_position = quarter_position as u64 * LOOKUP_SIZE as u64;
    let index = (lut_position / QUARTER_PHASE as u64) as usize;
    let remainder = lut_position % QUARTER_PHASE as u64;

    let value = if index >= LOOKUP_SIZE {
        SIN_LUT[LOOKUP_SIZE] as i64
    } else {
        let start = SIN_LUT[index] as i64;
        let difference = SIN_LUT[index + 1] as i64 - start;
        start + difference * remainder as i64 / QUARTER_PHASE as i64
    };

    if negative {
        -(value as i32)
    } else {
        value as i32
    }
}

pub(crate) fn cos(x: u32) -> i32 {
    const ONE_QUARTER_PHASE: u32 = 1 << 30;

    sin(x.wrapping_add(ONE_QUARTER_PHASE))
}
