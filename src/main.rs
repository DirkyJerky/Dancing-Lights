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
mod utils;

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
    DancingLights, IRLedPins, LedArrayPinsLeft, LedArrayPinsMiddle, LedArrayPinsRight,
    PhotoresistorPins, RGBLedPins,
};
use crate::utils::pwm_led::PWMLed;

// Setup the ADC, PIO, and DMA interrupts for further operations
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

/// Runs the Cyw43 wireless chip operations
#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

/// Checks the BOOTSEL button on a loop, reporting its state
#[embassy_executor::task]
async fn bootsel_polling(poller: BootselPoller<'static>) -> ! {
    poller.start_polling().await
}

/// Main entry point.
/// Performs the following
/// - Configures PIO communicates for SPI
/// - Loads the cyw43 firmware over the PIO channel
/// - Captures the necessary perhiperals for the system controllers
/// - Constructs the controllers and collects them into a `DancingLights` collection
/// - Delegates execution to [`crate::run::entry::run()`]
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

    // Setup the PWM outputs
    let (pin0_pwm, pin1_pwm) =
        PWMLed::new_ab(peripherals.PWM_SLICE0, peripherals.PIN_0, peripherals.PIN_1);
    let (pin2_pwm, pin3_pwm) =
        PWMLed::new_ab(peripherals.PWM_SLICE1, peripherals.PIN_2, peripherals.PIN_3);
    let (pin4_pwm, pin5_pwm) =
        PWMLed::new_ab(peripherals.PWM_SLICE2, peripherals.PIN_4, peripherals.PIN_5);
    let (pin6_pwm, pin7_pwm) =
        PWMLed::new_ab(peripherals.PWM_SLICE3, peripherals.PIN_6, peripherals.PIN_7);
    let (pin8_pwm, pin9_pwm) =
        PWMLed::new_ab(peripherals.PWM_SLICE4, peripherals.PIN_8, peripherals.PIN_9);
    let (pin10_pwm, pin11_pwm) = PWMLed::new_ab(
        peripherals.PWM_SLICE5,
        peripherals.PIN_10,
        peripherals.PIN_11,
    );
    let (pin12_pwm, pin13_pwm) = PWMLed::new_ab(
        peripherals.PWM_SLICE6,
        peripherals.PIN_12,
        peripherals.PIN_13,
    );
    let (pin14_pwm, pin15_pwm) = PWMLed::new_ab(
        peripherals.PWM_SLICE7,
        peripherals.PIN_14,
        peripherals.PIN_15,
    );

    let led_array_bottom = LedArrayPinsRight {
        red: pin0_pwm,
        yellow: pin1_pwm,
        green: pin2_pwm,
        blue: pin3_pwm,
        right_row: peripherals.PIN_18,
        left_row: peripherals.PIN_28,
    };

    let led_array_middle = LedArrayPinsMiddle {
        red: pin4_pwm,
        yellow: pin5_pwm,
        green: pin6_pwm,
        blue: pin7_pwm,
        right_row: peripherals.PIN_27,
        left_row: peripherals.PIN_22,
    };

    let led_array_top = LedArrayPinsLeft {
        red: pin8_pwm,
        yellow: pin9_pwm,
        green: pin10_pwm,
        blue: pin11_pwm,
        right_row: peripherals.PIN_20,
        left_row: peripherals.PIN_17,
    };

    let rgb_led = RGBLedPins {
        red: pin13_pwm,
        blue: pin14_pwm,
        green: pin15_pwm,
    };

    let ir_led = IRLedPins { vcc: pin12_pwm };

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
