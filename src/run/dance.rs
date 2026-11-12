use embassy_futures::join::{join, join4};
use embassy_rp::{clocks::RoscRng, peripherals::PWM_SLICE2};
use embassy_time::{Duration, Timer};

use crate::{
    cyw43_gpio::Cyw43_GPIO,
    ir_led::IRLed,
    led_array::{ArrayPin, LEDArray, PinState},
    photoresistor::Photoresistor,
    pinout::DancingLights,
    rgb_led::{ColorPin, RGBLed},
    utils::trig,
};

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

    let (_never, ..) = join4(
        blink(cyw43_control),
        read_photoresistor(photoresistor),
        strobe_rgb(rgb_led),
        join(
            flash_arrays(bottom_array, middle_array, top_array),
            fade_ir(ir_led),
        ),
    )
    .await;

    _never
}

/// Blink the onboard LED using the GPIO pin through the wireless chip
async fn blink(cyw43_control: &mut Cyw43_GPIO) -> ! {
    loop {
        cyw43_control.up(0).await;
        Timer::after(Duration::from_millis(50)).await;

        cyw43_control.down(0).await;
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
async fn fade_ir(ir: &mut IRLed<'static, PWM_SLICE2>) {
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
        let rand = rng.next_u32();

        rgb.set_pin(ColorPin::Red, rand & 1u32 << 0 != 0);
        rgb.set_pin(ColorPin::Green, rand & 1u32 << 1 != 0);
        rgb.set_pin(ColorPin::Blue, rand & 1u32 << 2 != 0);

        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Randomize the array states every second
async fn flash_arrays(
    bottom: &mut LEDArray<'static>,
    middle: &mut LEDArray<'static>,
    top: &mut LEDArray<'static>,
) -> ! {
    let mut rng = RoscRng;

    loop {
        let rand = rng.next_u32();

        for (array, master_offset) in [(&mut *bottom, 0), (&mut *middle, 8), (&mut *top, 16)] {
            let byte: u8 = ((rand >> master_offset) & 0xFF) as u8;
            let color_nibble: u8 = (byte >> 4) & 0x0F;
            let row_nibble: u8 = byte & 0x0F;

            for (pin, bit_offset) in [
                (ArrayPin::Red, 0),
                (ArrayPin::Yellow, 1),
                (ArrayPin::Green, 2),
                (ArrayPin::Blue, 3),
            ] {
                array.set_pin(
                    pin,
                    if color_nibble & 1u8 << bit_offset != 0 {
                        PinState::High
                    } else {
                        PinState::HighImpedance
                    },
                );
            }

            let (top_state, bottom_state) = match row_nibble {
                0..6 => (PinState::HighImpedance, PinState::Low),
                6..10 => (PinState::Low, PinState::HighImpedance),
                10..=16 => (PinState::Low, PinState::Low),
                _ => unreachable!(),
            };

            array.set_pin(ArrayPin::Top, top_state);
            array.set_pin(ArrayPin::Bottom, bottom_state);
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}
