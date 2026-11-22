//! Make the colors dance!
//!
//! Randomizes the output of all perpiherals, according
//! to the provided runner functions, which execute in parallel

use embassy_futures::join::{join, join4};
use embassy_rp::clocks::RoscRng;
use embassy_time::{Duration, Timer};

use crate::{
    cyw43_gpio::{Cyw43Gpio, Cyw43GpioPin},
    ir_led::IRLed,
    led_array::{ColorPin, GroundPin},
    photoresistor::Photoresistor,
    pinout::{DancingLights, LeftLEDArray, MiddleLEDArray, RightLEDArray},
    rgb_led::RGBLed,
    utils::trig,
};

pub async fn run(lights: &mut DancingLights<'static>) -> ! {
    let DancingLights {
        cyw43_control,
        photoresistor,
        right_array,
        middle_array,
        left_array,
        rgb_led,
        ir_led,
        bootsel_button: _,
    } = lights;

    let (_never, ..) = join4(
        blink(cyw43_control),
        read_photoresistor(photoresistor),
        strobe_rgb(rgb_led),
        join(
            flash_arrays(right_array, middle_array, left_array),
            fade_ir(ir_led),
        ),
    )
    .await;

    _never
}

/// Blink the onboard LED using the GPIO pin through the wireless chip
async fn blink(cyw43_control: &mut Cyw43Gpio) -> ! {
    loop {
        cyw43_control.up(Cyw43GpioPin::LED).await;
        Timer::after(Duration::from_millis(50)).await;

        cyw43_control.down(Cyw43GpioPin::LED).await;
        Timer::after(Duration::from_millis(950)).await;
    }
}

/// Read the photoresistor status in a loop
async fn read_photoresistor(photoresistor: &mut Photoresistor<'static>) -> ! {
    loop {
        match photoresistor.read().await {
            Err(e) => defmt::error!("{}", e),
            Ok(val) => defmt::info!("Photoresistor value: {}", val),
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}

/// Cycle the IR Led duty cycle through a sine wave
/// with a period of two seconds
///
/// Note: Gamma correction may be necessary
async fn fade_ir(ir: &mut IRLed<'static>) {
    const MIN: u16 = u16::MAX / 4;
    const MAX: u16 = u16::MAX / 4 * 3;

    let mut step = 0u32;
    let amplitude = (MAX - MIN) / 2;
    let midpoint = MIN + amplitude;

    loop {
        // 200 samples at 10 ms each gives a two-second cycle.
        let phase = step * (u32::MAX / 200);
        let sine = trig::sin(phase);

        // Scale the signed sine to roughly -65535..65535 before multiplying
        // by the duty-cycle amplitude, keeping the math within 32 bits.
        let normalized_sine = sine / 32_768;
        let duty_offset = normalized_sine * amplitude as i32 / u16::MAX as i32;
        let duty_cycle = (midpoint as i32 + duty_offset) as u16;

        ir.set_brightness(duty_cycle);

        step = (step + 1) % 200;
        Timer::after(Duration::from_millis(10)).await;
    }
}

/// Randomize the RGB Led every half second
async fn strobe_rgb(rgb: &mut RGBLed<'static>) -> ! {
    let mut rng = RoscRng;

    loop {
        rgb.set(
            rng.next_u32() as u16,
            rng.next_u32() as u16,
            rng.next_u32() as u16,
        );

        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Randomize the array states every second
async fn flash_arrays(
    right: &mut RightLEDArray<'static>,
    middle: &mut MiddleLEDArray<'static>,
    left: &mut LeftLEDArray<'static>,
) -> ! {
    let mut rng = RoscRng;

    macro_rules! flash_array {
        ($array:expr, $byte:expr) => {{
            let array = &mut *$array;
            let byte = $byte;
            let row_nibble = byte & 0x0F;
            let red_yellow = rng.next_u32();
            let green_blue = rng.next_u32();

            for (color, brightness) in [
                (ColorPin::Red, red_yellow as u16),
                (ColorPin::Yellow, (red_yellow >> 16) as u16),
                (ColorPin::Green, green_blue as u16),
                (ColorPin::Blue, (green_blue >> 16) as u16),
            ] {
                array.set_color_brightness(color, brightness);
            }

            let (left_enabled, right_enabled) = match row_nibble {
                0..6 => (false, true),
                6..10 => (true, false),
                10..=15 => (true, true),
                _ => unreachable!(),
            };
            array.set_ground(GroundPin::Left, left_enabled);
            array.set_ground(GroundPin::Right, right_enabled);
        }};
    }

    loop {
        let rand = rng.next_u32();
        flash_array!(right, rand as u8);
        flash_array!(middle, (rand >> 8) as u8);
        flash_array!(left, (rand >> 16) as u8);

        Timer::after(Duration::from_millis(250)).await;
    }
}
