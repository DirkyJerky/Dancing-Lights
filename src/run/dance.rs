use embassy_futures::join::{join_array, join4};
use embassy_rp::clocks::RoscRng;
use embassy_time::{Duration, Timer};

use crate::{
    cyw43_gpio::Cyw43_GPIO,
    led_array::{ArrayPin, LEDArray, PinState},
    photoresistor::Photoresistor,
    pinout::DancingLights,
    rgb_led::{ColorPin, RGBLed},
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
        flash_arrays(bottom_array, middle_array, top_array),
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

        for array in [&mut *bottom, &mut *middle, &mut *top] {
            array.set_pin(ArrayPin::Bottom, PinState::Low);
            array.set_pin(ArrayPin::Top, PinState::Low);
        }

        for (array, master_offset) in [(&mut *bottom, 0), (&mut *middle, 4), (&mut *top, 8)] {
            for (pin, bit_offset) in [
                (ArrayPin::Red, 0),
                (ArrayPin::Yellow, 1),
                (ArrayPin::Green, 2),
                (ArrayPin::Blue, 3),
            ] {
                array.set_pin(
                    pin,
                    if rand & 1u32 << (bit_offset + master_offset) != 0 {
                        PinState::High
                    } else {
                        PinState::Low
                    },
                );
            }
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}
