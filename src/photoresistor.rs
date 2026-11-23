//! Provides light-level readings from the photoresistor.

use embassy_rp::{
    Peri,
    adc::{self, Adc, AdcPin, Async, Channel, Config, Error},
    gpio::Pull,
    interrupt::typelevel::{ADC_IRQ_FIFO, Binding},
    peripherals::ADC,
};

/// A photoresistor read by an ADC controller
/// The photo resistor is set up as part of a voltage divider with a 10K resistor,
/// which normalizes the reading range to roughly (400..4000)
pub struct Photoresistor<'a> {
    adc: Adc<'a, Async>,
    channel: Channel<'a>,
}

impl<'a> Photoresistor<'a> {
    /// Construct a new photo resistor watcher,
    /// by configuring the provided ADC peripheral and channel
    pub fn new(
        adc: Peri<'a, ADC>,
        pin: Peri<'a, impl AdcPin + 'a>,
        irq: impl Binding<ADC_IRQ_FIFO, adc::InterruptHandler>,
    ) -> Self {
        Self {
            adc: Adc::new(adc, irq, Config::default()),
            channel: Channel::new_pin(pin, Pull::None),
        }
    }

    /// Read the current value of the photoresistor
    pub async fn read(&mut self) -> Result<u16, Error> {
        self.adc.read(&mut self.channel).await
    }
}
