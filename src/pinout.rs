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
    bootsel::BootselButton, cyw43_gpio::Cyw43Gpio, ir_led::IRLed, photoresistor::Photoresistor,
    rgb_led::RGBLed, utils::pwm_led::PWMLed,
};

use super::led_array::LEDArray;

use cyw43::Control;
use embassy_rp::{
    Peri,
    peripherals::{
        ADC, PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11,
        PIN_12, PIN_13, PIN_14, PIN_15, PIN_17, PIN_18, PIN_20, PIN_22, PIN_26, PIN_27, PIN_28,
        PWM_SLICE0, PWM_SLICE1, PWM_SLICE2, PWM_SLICE3, PWM_SLICE4, PWM_SLICE5, PWM_SLICE6,
        PWM_SLICE7,
    },
};

/// Right LED array: color channels on GPIO 0 through 3.
pub type RightLEDArray<'a> =
    LEDArray<'a, PWM_SLICE0, PIN_0, PWM_SLICE0, PIN_1, PWM_SLICE1, PIN_2, PWM_SLICE1, PIN_3>;

/// Middle LED array: color channels on GPIO 4 through 7.
pub type MiddleLEDArray<'a> =
    LEDArray<'a, PWM_SLICE2, PIN_4, PWM_SLICE2, PIN_5, PWM_SLICE3, PIN_6, PWM_SLICE3, PIN_7>;

/// Left LED array: color channels on GPIO 8 through 11.
pub type LeftLEDArray<'a> =
    LEDArray<'a, PWM_SLICE4, PIN_8, PWM_SLICE4, PIN_9, PWM_SLICE5, PIN_10, PWM_SLICE5, PIN_11>;

pub struct LedArrayPinsRight {
    pub red: PWMLed<'static, PWM_SLICE0, PIN_0>,
    pub yellow: PWMLed<'static, PWM_SLICE0, PIN_1>,
    pub green: PWMLed<'static, PWM_SLICE1, PIN_2>,
    pub blue: PWMLed<'static, PWM_SLICE1, PIN_3>,
    pub right_row: Peri<'static, PIN_18>,
    pub left_row: Peri<'static, PIN_28>,
}

impl<'a> Into<RightLEDArray<'a>> for LedArrayPinsRight {
    fn into(self) -> RightLEDArray<'a> {
        LEDArray::new(
            self.red,
            self.yellow,
            self.green,
            self.blue,
            self.right_row,
            self.left_row,
        )
    }
}

pub struct LedArrayPinsMiddle {
    pub red: PWMLed<'static, PWM_SLICE2, PIN_4>,
    pub yellow: PWMLed<'static, PWM_SLICE2, PIN_5>,
    pub green: PWMLed<'static, PWM_SLICE3, PIN_6>,
    pub blue: PWMLed<'static, PWM_SLICE3, PIN_7>,
    pub right_row: Peri<'static, PIN_27>,
    pub left_row: Peri<'static, PIN_22>,
}

impl<'a> Into<MiddleLEDArray<'a>> for LedArrayPinsMiddle {
    fn into(self) -> MiddleLEDArray<'a> {
        LEDArray::new(
            self.red,
            self.yellow,
            self.green,
            self.blue,
            self.right_row,
            self.left_row,
        )
    }
}

pub struct LedArrayPinsLeft {
    pub red: PWMLed<'static, PWM_SLICE4, PIN_8>,
    pub yellow: PWMLed<'static, PWM_SLICE4, PIN_9>,
    pub green: PWMLed<'static, PWM_SLICE5, PIN_10>,
    pub blue: PWMLed<'static, PWM_SLICE5, PIN_11>,
    pub right_row: Peri<'static, PIN_20>,
    pub left_row: Peri<'static, PIN_17>,
}

impl<'a> Into<LeftLEDArray<'a>> for LedArrayPinsLeft {
    fn into(self) -> LeftLEDArray<'a> {
        LEDArray::new(
            self.red,
            self.yellow,
            self.green,
            self.blue,
            self.right_row,
            self.left_row,
        )
    }
}

pub struct RGBLedPins {
    pub red: PWMLed<'static, PWM_SLICE6, PIN_13>,
    pub blue: PWMLed<'static, PWM_SLICE7, PIN_14>,
    pub green: PWMLed<'static, PWM_SLICE7, PIN_15>,
}

impl<'a> Into<RGBLed<'a>> for RGBLedPins {
    fn into(self) -> RGBLed<'a> {
        RGBLed::new(self.red, self.blue, self.green)
    }
}

pub struct IRLedPins {
    pub vcc: PWMLed<'static, PWM_SLICE6, PIN_12>,
}

impl<'a> Into<IRLed<'a>> for IRLedPins {
    fn into(self) -> IRLed<'a> {
        IRLed::new(self.vcc)
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
    pub cyw43_control: Cyw43Gpio,
    pub right_array: RightLEDArray<'a>,
    pub middle_array: MiddleLEDArray<'a>,
    pub left_array: LeftLEDArray<'a>,
    pub rgb_led: RGBLed<'a>,
    pub ir_led: IRLed<'a>,
    pub photoresistor: Photoresistor<'a>,
    pub bootsel_button: BootselButton,
}

impl<'a> DancingLights<'a> {
    pub fn new(
        cyw43_control: Control<'static>,
        right_array: LedArrayPinsRight,
        middle_array: LedArrayPinsMiddle,
        left_array: LedArrayPinsLeft,
        rgb_led: RGBLedPins,
        ir_led: IRLedPins,
        photoresistor: PhotoresistorPins,
        bootsel_button: BootselButton,
    ) -> DancingLights<'a> {
        DancingLights {
            cyw43_control: Cyw43Gpio::new(cyw43_control),
            right_array: right_array.into(),
            middle_array: middle_array.into(),
            left_array: left_array.into(),
            rgb_led: rgb_led.into(),
            ir_led: ir_led.into(),
            photoresistor: photoresistor.into(),
            bootsel_button,
        }
    }

    pub async fn reset(&mut self) {
        self.cyw43_control.reset().await;
        self.right_array.reset();
        self.middle_array.reset();
        self.left_array.reset();
        self.rgb_led.reset();
        self.ir_led.reset();
    }
}
