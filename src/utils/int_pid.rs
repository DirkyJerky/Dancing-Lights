//!  A Simple PID controller,
//!  implemented in fixed 32-bit arithmetic

use embassy_time::Instant;
use fixed::{Saturating, types::extra::U24};

// Fractional type with 8 bit integer component and 24 bit fractional component
pub type N = fixed::FixedI32<U24>;
// Its saturating version, used for internal calculations
type SN = Saturating<N>;

/// PID Limits
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    low: N,
    high: N,
}

impl Limits {
    /// Construct a new Limits.
    ///
    /// It is required that low < high
    ///
    /// Returns Some(limits) if low < high
    /// Returns None if low >= high
    pub const fn new(low: N, high: N) -> Option<Self> {
        // Uses .to_bits() to work in the const context
        if low.to_bits() >= high.to_bits() {
            None
        } else {
            Some(Self { low, high })
        }
    }

    /// Return the low limit
    pub const fn low(&self) -> N {
        self.low
    }

    /// Return the high limit
    pub const fn high(&self) -> N {
        self.high
    }

    /// Clamp a value to the bounds represented by this limit
    pub fn clamp(&self, value: SN) -> SN {
        value.clamp(Saturating(self.low), Saturating(self.high))
    }
}

/// PID Gains
#[derive(Debug, Clone, Copy)]
pub struct Gains {
    kp: N,
    ki: N,
    kd: N,
}

impl Gains {
    pub const fn new(kp: N, ki: N, kd: N) -> Self {
        Self { kp, ki, kd }
    }

    pub const fn kp(&self) -> SN {
        Saturating(self.kp)
    }

    pub const fn ki(&self) -> SN {
        Saturating(self.ki)
    }

    pub const fn kd(&self) -> SN {
        Saturating(self.kd)
    }
}

struct PIDState {
    pub integral_agg: SN,
    pub last_measurement_value: Option<SN>,
    pub last_measurement_time: Option<Instant>,
}

impl Default for PIDState {
    fn default() -> Self {
        Self {
            integral_agg: SN::ZERO,
            last_measurement_value: None,
            last_measurement_time: None,
        }
    }
}

impl PIDState {
    fn new() -> Self {
        Default::default()
    }
}

pub struct PID {
    integral_limits: Limits,
    limits: Limits,
    gains: Gains,
    state: PIDState,
    setpoint: SN,
}

impl PID {
    pub fn new(setpoint: N, integral_limits: Limits, limits: Limits, gains: Gains) -> Self {
        Self {
            setpoint: Saturating(setpoint),
            integral_limits,
            limits,
            gains,
            state: PIDState::new(),
        }
    }

    pub fn setpoint(&self) -> N {
        self.setpoint.0
    }

    pub fn update(&mut self, measurement: N) -> N {
        let (output, _) = self.update_with_residues(measurement);
        output
    }

    pub fn update_with_residues(&mut self, measurement: N) -> (N, (N, N, N)) {
        let measurement = Saturating(measurement);
        let now = Instant::now();
        // The microseconds that elapsed since the last call.
        // Is Some(...) if there has been a previous call to compare to
        // Is None if there has not been a previous call to compare to
        //
        // Note: Normally, duration_since would panic if `last_time` comes after `now`.
        //
        // However, that is not an issue here, as
        // embassy-time guarentees that Instant::now() is monotonic.
        //
        // Therefore, as the previous `last_time` was guarenteed to have been constructed
        // before `now`, the following call is safe.
        let elapsed_micros: Option<u64> = self
            .state
            .last_measurement_time
            .map(|last_time| now.duration_since(last_time).as_micros());

        // Positive seconds elapsed.
        // Some(...) if a non-zero duration has elapsed
        // None if no previous call or the duration is zero
        let dt_seconds: Option<SN> = elapsed_micros.and_then(|micros| {
            // Define constants
            const Q24: u64 = 1 << 24;
            const MICROS_PER_SECOND: u64 = 1_000_000;

            // Clamp to the maximum value
            let max_micros = (i32::MAX as u64 * MICROS_PER_SECOND) / Q24;
            let micros = micros.min(max_micros);

            // Convert
            let raw = micros * Q24 / MICROS_PER_SECOND;
            let n = SN::from_bits(raw as i32);

            if n == SN::ZERO { None } else { Some(n) }
        });

        let error = self.setpoint - measurement;

        let p = error * self.gains.kp();
        let i = match dt_seconds {
            Some(dt) => self.state.integral_agg + error * dt * self.gains.ki(),
            None => self.state.integral_agg,
        };
        let i = self.integral_limits.clamp(i);
        let d = match (self.state.last_measurement_value, dt_seconds) {
            (Some(last_measurement), Some(dt)) => {
                -self.gains.kd() * (measurement - last_measurement) / dt
            }
            _ => SN::ZERO,
        };

        let output = self.limits.clamp(p + i + d);

        self.state.integral_agg = i;
        self.state.last_measurement_value = Some(measurement);
        self.state.last_measurement_time = Some(now);
        (output.0, (p.0, i.0, d.0))
    }
}
