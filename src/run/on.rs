//! Turns all peripherals on
//!

use core::future::pending;

use crate::{
    cyw43_gpio::Cyw43GpioPin,
    led_array::{ColorPin, GroundPin},
    pinout::DancingLights,
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
        bootsel_button,
    } = lights;

    cyw43_control.up(Cyw43GpioPin::LED).await;
    macro_rules! turn_on {
        ($array:expr) => {{
            let array = &mut *$array;
            array.set_color_brightness(ColorPin::Red, u16::MAX);
            array.set_color_brightness(ColorPin::Yellow, u16::MAX);
            array.set_color_brightness(ColorPin::Green, u16::MAX);
            array.set_color_brightness(ColorPin::Blue, u16::MAX);
            array.set_ground(GroundPin::Left, true);
            array.set_ground(GroundPin::Right, true);
        }};
    }

    turn_on!(right_array);
    turn_on!(middle_array);
    turn_on!(left_array);

    rgb_led.up();
    ir_led.up();

    pending::<()>().await;
    unreachable!()
}
