//! Pinout configuration for the board setup
//!
//! LED Array
//! ---------
//!
//! Set 1 (Bottom of board):
//! Red VCC => GPIO 0
//! Yellow VCC => GPIO 1
//! Green VCC => GPIO 2
//! Blue VCC => GPIO 3
//! Bottom row GND => GPIO 18
//! Top row GND => GPIO 28
//!
//! Set 2 (Middle):
//! Red VCC => GPIO 4
//! Yellow VCC => GPIO 5
//! Green VCC => GPIO 6
//! Blue VCC => GPIO 7
//! Bottom row GND => GPIO 27
//! Top row GND => GPIO 22
//!
//! Set 3 (Top):
//! Red VCC => GPIO 8
//! Yellow VCC => GPIO 9
//! Green VCC => GPIO 10
//! Blue VCC => GPIO 11
//! Bottom row GND => GPIO 20
//! Top row GND => GPIO 17
//!
//!
//! RGB LED
//! -------
//!
//! Red (1 SURK) => GPIO 12
//! Float (2 GND) => GPIO 13
//! Blue (3 QB-D) => GPIO 14
//! Green (4 ZG) => GPIO 15
//!
//!
//! IR LED
//! ------
//! VCC => GPIO 21
//! Float => GPIO 19
//!
//!
//! Photoresistor
//! -------------
//! ADC_VREF (35)
//! ADC_0 / GPIO 26
//!
//! Divided with a 10k to GND
//!
//!
//!

use crate::{
    bootsel::BootselButton, cyw43_gpio::Cyw43_GPIO, ir_led::IRLed, photoresistor::Photoresistor,
    rgb_led::RGBLed,
};

use super::led_array::LEDArray;

use cyw43::Control;
use embassy_rp::{
    Peri,
    peripherals::{
        ADC, PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11,
        PIN_12, PIN_13, PIN_14, PIN_15, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_26,
        PIN_27, PIN_28,
    },
};

pub struct LedArrayPinsBottom {
    pub red: Peri<'static, PIN_0>,
    pub yellow: Peri<'static, PIN_1>,
    pub green: Peri<'static, PIN_2>,
    pub blue: Peri<'static, PIN_3>,
    pub bottom_row: Peri<'static, PIN_18>,
    pub top_row: Peri<'static, PIN_28>,
}

impl<'a> Into<LEDArray<'a>> for LedArrayPinsBottom {
    fn into(self) -> LEDArray<'a> {
        LEDArray::new(
            self.red,
            self.yellow,
            self.green,
            self.blue,
            self.bottom_row,
            self.top_row,
        )
    }
}

pub struct LedArrayPinsMiddle {
    pub red: Peri<'static, PIN_4>,
    pub yellow: Peri<'static, PIN_5>,
    pub green: Peri<'static, PIN_6>,
    pub blue: Peri<'static, PIN_7>,
    pub bottom_row: Peri<'static, PIN_27>,
    pub top_row: Peri<'static, PIN_22>,
}

impl<'a> Into<LEDArray<'a>> for LedArrayPinsMiddle {
    fn into(self) -> LEDArray<'a> {
        LEDArray::new(
            self.red,
            self.yellow,
            self.green,
            self.blue,
            self.bottom_row,
            self.top_row,
        )
    }
}

pub struct LedArrayPinsTop {
    pub red: Peri<'static, PIN_8>,
    pub yellow: Peri<'static, PIN_9>,
    pub green: Peri<'static, PIN_10>,
    pub blue: Peri<'static, PIN_11>,
    pub bottom_row: Peri<'static, PIN_20>,
    pub top_row: Peri<'static, PIN_17>,
}

impl<'a> Into<LEDArray<'a>> for LedArrayPinsTop {
    fn into(self) -> LEDArray<'a> {
        LEDArray::new(
            self.red,
            self.yellow,
            self.green,
            self.blue,
            self.bottom_row,
            self.top_row,
        )
    }
}

pub struct RGBLedPins {
    pub red: Peri<'static, PIN_12>,
    pub float: Peri<'static, PIN_13>,
    pub blue: Peri<'static, PIN_14>,
    pub green: Peri<'static, PIN_15>,
}

impl<'a> Into<RGBLed<'a>> for RGBLedPins {
    fn into(self) -> RGBLed<'a> {
        RGBLed::new(self.red, self.float, self.blue, self.green)
    }
}

pub struct IRLedPins {
    pub vcc: Peri<'static, PIN_21>,
    pub float: Peri<'static, PIN_19>,
}

impl<'a> Into<IRLed<'a>> for IRLedPins {
    fn into(self) -> IRLed<'a> {
        IRLed::new(self.vcc, self.float)
    }
}

pub struct PhotoresistorPins {
    pub adc: Peri<'static, ADC>,
    pub pin: Peri<'static, PIN_26>,
    pub irq: crate::Irqs,
}

impl<'a> Into<Photoresistor<'a>> for PhotoresistorPins {
    fn into(self) -> Photoresistor<'a> {
        Photoresistor::new(self.adc, self.pin, self.irq)
    }
}

pub struct DancingLights<'a> {
    pub cyw43_control: Cyw43_GPIO,
    pub bottom_array: LEDArray<'a>,
    pub middle_array: LEDArray<'a>,
    pub top_array: LEDArray<'a>,
    pub rgb_led: RGBLed<'a>,
    pub ir_led: IRLed<'a>,
    pub photoresistor: Photoresistor<'a>,
    pub bootsel_button: BootselButton,
}

impl<'a> DancingLights<'a> {
    pub fn new(
        cyw43_control: Control<'static>,
        bottom_array: LedArrayPinsBottom,
        middle_array: LedArrayPinsMiddle,
        top_array: LedArrayPinsTop,
        rgb_led: RGBLedPins,
        ir_led: IRLedPins,
        photoresistor: PhotoresistorPins,
        bootsel_button: BootselButton,
    ) -> DancingLights<'a> {
        DancingLights {
            cyw43_control: Cyw43_GPIO::new(cyw43_control),
            bottom_array: bottom_array.into(),
            middle_array: middle_array.into(),
            top_array: top_array.into(),
            rgb_led: rgb_led.into(),
            ir_led: ir_led.into(),
            photoresistor: photoresistor.into(),
            bootsel_button,
        }
    }

    pub async fn reset(&mut self) {
        self.cyw43_control.reset().await;
        self.bottom_array.reset();
        self.middle_array.reset();
        self.top_array.reset();
        self.rgb_led.reset();
        self.ir_led.reset();
    }
}
