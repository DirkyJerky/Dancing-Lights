use core::future::pending;

use crate::{
    led_array::{ArrayPin, PinState},
    pinout::DancingLights,
    rgb_led::ColorPin,
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

    cyw43_control.up(0).await;
    for array in [&mut *bottom_array, &mut *middle_array, &mut *top_array] {
        array.set_pin(ArrayPin::Red, PinState::High);
        array.set_pin(ArrayPin::Yellow, PinState::High);
        array.set_pin(ArrayPin::Green, PinState::High);
        array.set_pin(ArrayPin::Blue, PinState::High);
        array.set_pin(ArrayPin::Bottom, PinState::Low);
        array.set_pin(ArrayPin::Top, PinState::Low);
    }

    rgb_led.set_pin(ColorPin::Red, true);
    rgb_led.set_pin(ColorPin::Green, true);
    rgb_led.set_pin(ColorPin::Blue, true);

    ir_led.up();

    pending::<()>().await;
    unreachable!()
}
