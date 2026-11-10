use embassy_rp::{
    Peri,
    adc::{self, Adc, AdcPin, Async, Channel, Config, Error},
    gpio::Pull,
    interrupt::typelevel::{ADC_IRQ_FIFO, Binding},
    peripherals::ADC,
};

pub struct Photoresistor<'a> {
    adc: Adc<'a, Async>,
    channel: Channel<'a>,
}

impl<'a> Photoresistor<'a> {
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

    pub async fn read(&mut self) -> Result<u16, Error> {
        self.adc.read(&mut self.channel).await
    }
}
