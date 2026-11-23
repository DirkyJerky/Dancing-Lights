//! Provides control of the Pico W's GPIO signals connected to the CYW43 chip.

use cyw43::Control;

/// Cyw43's addressable GPIO pins, wired for distinct purposes
pub enum Cyw43GpioPin {
    /// Onboard LED
    LED,
    /// RT6154 Power-Select
    SMPSPowerSelect,
    /// VBUS Sense
    VBUSSense,
}

impl Cyw43GpioPin {
    fn addr(&self) -> u8 {
        match self {
            Self::LED => 0,
            Self::SMPSPowerSelect => 1,
            Self::VBUSSense => 2,
        }
    }
}

/// Controller for the Cyw43's GPIO pins
pub struct Cyw43Gpio {
    control: Control<'static>,
}

impl Cyw43Gpio {
    /// Construct the controller, using an initialized control interface
    pub fn new(control: Control<'static>) -> Self {
        Self { control }
    }

    /// Sets a pin to high.
    /// Only supports Pin values
    pub async fn up(&mut self, pin: Cyw43GpioPin) {
        self.control.gpio_set(pin.addr(), true).await;
    }

    pub async fn down(&mut self, pin: Cyw43GpioPin) {
        self.control.gpio_set(pin.addr(), false).await;
    }

    pub async fn reset(&mut self) {
        self.down(Cyw43GpioPin::LED).await;
        self.down(Cyw43GpioPin::SMPSPowerSelect).await;
        self.down(Cyw43GpioPin::VBUSSense).await;
    }
}
