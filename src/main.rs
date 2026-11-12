#![no_std]
#![no_main]

mod bootsel;
mod cyw43_gpio;
mod ir_led;
mod led_array;
mod photoresistor;
mod pinout;
mod rgb_led;
mod run;

use cyw43::aligned_bytes;
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::{
    bind_interrupts, dma,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, DMA_CH1, PIO0},
};
use embassy_time::Duration;
use panic_probe as _;
use static_cell::StaticCell;

use crate::bootsel::{BootselButton, BootselPoller};
use crate::pinout::{
    DancingLights, IRLedPins, LedArrayPinsBottom, LedArrayPinsMiddle, LedArrayPinsTop,
    PhotoresistorPins, RGBLedPins,
};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(spawner: Spawner) {
    defmt::info!("Initializing...");

    let peripherals = embassy_rp::init(Default::default());

    let fw = aligned_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = aligned_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let nvram = aligned_bytes!("../cyw43-firmware/nvram_rp2040.bin");

    let pwr = Output::new(peripherals.PIN_23, Level::Low);
    let cs = Output::new(peripherals.PIN_25, Level::High);
    let mut pio = Pio::new(peripherals.PIO0, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        peripherals.PIN_24,
        peripherals.PIN_29,
        dma::Channel::new(peripherals.DMA_CH0, Irqs),
    );

    let start = embassy_time::Instant::now();
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    spawner.spawn(cyw43_task(runner).unwrap());
    defmt::debug!("cyw43::new took {} ms", start.elapsed().as_millis());

    let start = embassy_time::Instant::now();
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    defmt::debug!("control.init took {} ms", start.elapsed().as_millis());

    let led_array_bottom = LedArrayPinsBottom {
        red: peripherals.PIN_0,
        yellow: peripherals.PIN_1,
        green: peripherals.PIN_2,
        blue: peripherals.PIN_3,
        bottom_row: peripherals.PIN_18,
        top_row: peripherals.PIN_28,
    };

    let led_array_middle = LedArrayPinsMiddle {
        red: peripherals.PIN_4,
        yellow: peripherals.PIN_5,
        green: peripherals.PIN_6,
        blue: peripherals.PIN_7,
        bottom_row: peripherals.PIN_27,
        top_row: peripherals.PIN_22,
    };

    let led_array_top = LedArrayPinsTop {
        red: peripherals.PIN_8,
        yellow: peripherals.PIN_9,
        green: peripherals.PIN_10,
        blue: peripherals.PIN_11,
        bottom_row: peripherals.PIN_20,
        top_row: peripherals.PIN_17,
    };

    let rgb_led = RGBLedPins {
        red: peripherals.PIN_12,
        float: peripherals.PIN_13,
        blue: peripherals.PIN_14,
        green: peripherals.PIN_15,
    };

    let ir_led = IRLedPins {
        vcc: peripherals.PIN_21,
        float: peripherals.PIN_19,
    };

    let photoresist = PhotoresistorPins {
        adc: peripherals.ADC,
        pin: peripherals.PIN_26,
        irq: Irqs,
    };

    let (bootsel_button, bootsel_poller) =
        BootselButton::new(peripherals.BOOTSEL, Duration::from_millis(50));

    spawner.spawn(bootsel_polling(bootsel_poller).unwrap());

    let lights: DancingLights<'static> = DancingLights::new(
        control,
        led_array_bottom,
        led_array_middle,
        led_array_top,
        rgb_led,
        ir_led,
        photoresist,
        bootsel_button,
    );

    defmt::info!("Initialized.");

    run::run(spawner, lights).await;
}

#[embassy_executor::task]
async fn bootsel_polling(poller: BootselPoller<'static>) -> ! {
    poller.start_polling().await
}
