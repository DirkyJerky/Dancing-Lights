use cyw43::Control;

/// Cyw43's addressable GPIO pins, wired for distinct purposes
pub enum Cyw43GpioPin {
    /// Onboard LED
    LED,
    /// RT6154 Power-Select
    SMPS_Power_Select,
    /// VBUS Sense
    VBUS_Sense,
}

impl Cyw43GpioPin {
    fn addr(&self) -> u8 {
        match self {
            Self::LED => 0,
            Self::SMPS_Power_Select => 1,
            Self::VBUS_Sense => 2,
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
        self.down(Cyw43GpioPin::SMPS_Power_Select).await;
        self.down(Cyw43GpioPin::VBUS_Sense).await;
    }
}
