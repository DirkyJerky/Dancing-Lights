//! Runs a PID controller, displaying
//! all the P/I/D values during operation on the LED arrays
//!
//! The plant is the IR Led,
//! and the sensor is the Photoresist.
//!
//! This controller attempts to set the photoresist value
//! to a reading slightly above the passive light level in my apartment.
//!
//! By phyiscally varying the distance of the IR Led to the photoresist, this controller
//! will demonstrate its behavior via the realtime response guages.

use core::future::pending;

use embassy_futures::join::join;
use embassy_rp::spinlock_mutex::SpinlockRawMutex;
use embassy_sync::watch::Watch;
use embassy_time::{Duration, Timer};
use fixed::Saturating;

use crate::{
    ir_led::IRLed,
    led_array::{ArrayPin, LEDArray, PinState},
    photoresistor::Photoresistor,
    pinout::DancingLights,
    utils::int_pid::{Gains, Limits, N, PID, SN},
};

static SPINLOCK_ID: usize = 1;
static WATCHER_COUNT: usize = 1;

type MUTEX = SpinlockRawMutex<SPINLOCK_ID>;

static WATCH: Watch<MUTEX, u16, WATCHER_COUNT> = Watch::new();

// Photo resistor readings are typically in the 400 - 4000 range.
// We are going to scale down that range to fit in the range of N::MAX (roughly ~128)
// By dividing by 32, the range becomes roughly 12 - 125
const SCALE: u16 = 32;
fn encode_resistor_reading(val: u16) -> SN {
    let whole = val / SCALE;
    let remainder = val % SCALE;

    Saturating(N::saturating_from_num(whole))
        + Saturating(N::saturating_from_num(remainder)) / Saturating(N::saturating_from_num(SCALE))
}

pub async fn run(lights: &mut DancingLights<'static>) -> ! {
    let DancingLights {
        cyw43_control,
        photoresistor,
        bottom_array,
        middle_array,
        top_array,
        rgb_led,
        ir_led,
        bootsel_button,
    } = lights;

    // Gain values that are intentionally underdamped,
    // to show intentional oscillation
    const UNDERDAMPED_GAINS: Gains = {
        const KP: N = N::lit("0.001");
        const KI: N = N::lit("0.1");
        const KD: N = N::lit("0.0001");
        Gains::new(KP, KI, KD)
    };

    // Gain values that are calibrated
    // to respond efficiently without overshooting
    const CALIBRATED_GAINS: Gains = {
        const KP: N = N::lit("0.0005");
        const KI: N = N::lit("0.0");
        const KD: N = N::lit("0.00");
        Gains::new(KP, KI, KD)
    };

    const GAINS: Gains = CALIBRATED_GAINS;

    /// Integral limits for internal PID operation
    const I_LIMITS: Limits = Limits::new(N::lit("-1.0"), N::lit("1.0")).unwrap();
    /// Output limits for the PID controller, mapped directly to IR Led brightness control
    const OUTPUT_LIMITS: Limits = Limits::new(N::lit("0.0"), N::lit("1.0")).unwrap();
    /// The min/max values used for the Integral guage display
    const I_GAUGE_LIMITS: Limits = I_LIMITS;
    /// The target reading on the sensor
    /// Myoom light level reads ~1700 at night, ~2500 at day
    /// So lets target a bit higher than that
    const TARGET: u16 = 2750;

    // Calculate the P guage limits
    const ADC_MAX: u16 = 4095;
    const MIN_SAMPLE_PERIOD_SECS: SN = Saturating(N::lit("0.01"));
    const MIN_D_GAUGE_SPAN: SN = Saturating(N::lit("0.1"));
    let setpoint: SN = encode_resistor_reading(TARGET);

    // Derive the P gauge's scale from the min/max of the sensor range using
    // the controller equation: P = (setpoint - measurement) * kp.
    let p_at_adc_min = (setpoint - encode_resistor_reading(0)) * GAINS.kp();
    let p_at_adc_max = (setpoint - encode_resistor_reading(ADC_MAX)) * GAINS.kp();
    let p_gauge_limits = Limits::new(
        p_at_adc_min.min(p_at_adc_max).0,
        p_at_adc_min.max(p_at_adc_max).0,
    )
    .expect("ADC range must produce a non-zero P range");

    // Calculate the D guage limits
    //
    // D = -kd * delta(measurement) / dt. Use the full ADC range and the
    // shortest expected interval to estimate the max value.
    let sensor_span = encode_resistor_reading(ADC_MAX) - encode_resistor_reading(0);
    let d_max = (GAINS.kd() * sensor_span / MIN_SAMPLE_PERIOD_SECS)
        .abs()
        .max(MIN_D_GAUGE_SPAN);
    let d_gauge_limits =
        Limits::new(-d_max.0, d_max.0).expect("D gauge span must be greater than zero");

    // Construct the PID controller
    let pid: PID = PID::new(setpoint.0, I_LIMITS, OUTPUT_LIMITS, GAINS);

    // Set up the guages
    let mut guage_p: ArrayGuage = ArrayGuage::new(p_gauge_limits, top_array);
    let mut guage_i: ArrayGuage = ArrayGuage::new(I_GAUGE_LIMITS, middle_array);
    let mut guage_d: ArrayGuage = ArrayGuage::new(d_gauge_limits, bottom_array);

    // Run the feedback loop
    join(
        watch_photoresist(pid, ir_led, &mut guage_p, &mut guage_i, &mut guage_d),
        poll_photoresist(photoresistor),
    )
    .await;

    pending::<()>().await;
    unreachable!()
}

/// Converts an LED array into a display guage, showing a values range within an expected Limit range
struct ArrayGuage<'b, 'peri> {
    limits: Limits,
    array: &'b mut LEDArray<'peri>,
}

impl<'b, 'peri> ArrayGuage<'b, 'peri> {
    pub fn new(limits: Limits, array: &'b mut LEDArray<'peri>) -> Self {
        array.set_pin(ArrayPin::Bottom, PinState::Low);
        array.set_pin(ArrayPin::Top, PinState::Low);
        Self { limits, array }
    }

    /// Set the guage level according to the provided value
    pub fn set(&mut self, val: N) {
        let low = self.limits.low();
        let high = self.limits.high();
        let diff = high - low;
        let step = diff / 5;

        let thresholds = [
            (ArrayPin::Red, low + step),
            (ArrayPin::Yellow, low + 2 * step),
            (ArrayPin::Green, low + 3 * step),
            (ArrayPin::Blue, low + 4 * step),
        ];

        for (pin, threshold) in thresholds {
            if val > threshold {
                self.array.set_pin(pin, PinState::High);
            } else {
                self.array.set_pin(pin, PinState::HighImpedance);
            }
        }
    }
}

/// Watch the photoresistor values,
/// updating the PID controller and setting the IR LED brightness according
/// to the controller's response
///
/// Also updates the ArrayGuages with the PID's component values
async fn watch_photoresist<'a>(
    mut pid: PID,
    ir_led: &mut IRLed<'a>,
    guage_p: &mut ArrayGuage<'_, 'a>,
    guage_i: &mut ArrayGuage<'_, 'a>,
    guage_d: &mut ArrayGuage<'_, 'a>,
) {
    // This reciever slot will be recovered when this future goes out of scope.
    // So, we dont need to worry about exceeding the reciever count
    let mut reciever = WATCH.receiver().expect("Reciever should be available.");

    loop {
        let val = reciever.changed().await;

        let (output, (p, i, d)) = pid.update_with_residues(encode_resistor_reading(val).0);

        // Scale 0.0..1.0 output to 0..u16::MAX
        let raw = output.to_bits().clamp(0, 1 << 24) as u32;
        let brightness = (raw >> 8).min(u16::MAX as u32) as u16;

        ir_led.set_duty_cycle(brightness);
        guage_p.set(p);
        guage_i.set(i);
        guage_d.set(d);

        defmt::debug!("sensor={} p={} i={} d={} output={}", val, p, i, d, output,);

        ()
    }
}

/// Report the photo resistor's value
async fn poll_photoresist<'a>(photoresist: &mut Photoresistor<'a>) {
    let sender = WATCH.sender();

    loop {
        if let Ok(val) = photoresist.read().await {
            sender.send(val);
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}
