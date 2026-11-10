use embassy_executor::Spawner;

use embassy_rp::clocks::RoscRng;
use embassy_time::{Duration, Timer};

use crate::{
    cyw43_gpio::Cyw43_GPIO,
    led_array::{ArrayPin, LEDArray, PinState},
    photoresistor::Photoresistor,
    pinout::DancingLights,
    rgb_led::{ColorPin, RGBLed},
};

pub async fn run(spawner: Spawner, lights: DancingLights<'static>) -> ! {
    let DancingLights {
        cyw43_control,
        photoresistor,
        bottom_array,
        middle_array,
        top_array,
        rgb_led,
        ir_led,
    } = lights;

    spawner.spawn(blink(cyw43_control).unwrap());
    spawner.spawn(read_photoresistor(photoresistor).unwrap());
    spawner.spawn(strobe_rgb(rgb_led).unwrap());
    spawner.spawn(flash_arrays(bottom_array, middle_array, top_array).unwrap());

    // Keep the unused drivers alive rather than dropping and deconfiguring their pins.
    let _remaining_hardware = (ir_led);
    core::future::pending::<()>().await;
    unreachable!()
}

/// Blink the onboard LED using the GPIO pin through the wireless chip
#[embassy_executor::task]
async fn blink(mut cyw43_control: Cyw43_GPIO) -> ! {
    loop {
        // defmt::info!("LED on!");
        cyw43_control.up(0).await;
        Timer::after(Duration::from_millis(50)).await;

        // defmt::info!("LED off!");
        cyw43_control.down(0).await;
        Timer::after(Duration::from_millis(950)).await;
    }
}

/// Read the photoresistor status in a loop
#[embassy_executor::task]
async fn read_photoresistor(mut photoresistor: Photoresistor<'static>) -> ! {
    loop {
        match photoresistor.read().await {
            Err(e) => defmt::error!("{}", e),
            Ok(val) => defmt::info!("Photoresistor value: {}", val),
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}

/// Randomize the RGB Led every half second
#[embassy_executor::task]
async fn strobe_rgb(mut rgb: RGBLed<'static>) {
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
#[embassy_executor::task]
async fn flash_arrays(
    mut bottom: LEDArray<'static>,
    mut middle: LEDArray<'static>,
    mut top: LEDArray<'static>,
) {
    let mut rng = RoscRng;

    loop {
        let rand = rng.next_u32();

        for array in [&mut bottom, &mut middle, &mut top] {
            array.set_pin(ArrayPin::Bottom, PinState::Low);
            array.set_pin(ArrayPin::Top, PinState::Low);
        }

        for (array, master_offset) in [(&mut bottom, 0), (&mut middle, 4), (&mut top, 8)] {
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
