//! Main execution loop runner
//!
//! Runs after all component initalization has been performed.

use embassy_executor::Spawner;
use embassy_futures::select::select;

use crate::{bootsel::ButtonChangeState, pinout::DancingLights};

use super::dance::run as dance;
use super::feedback::run as feedback;
use super::off::run as off;
use super::on::run as on;

/// The runtime modes supported by this runner
#[derive(Debug, Clone, Copy)]
enum Mode {
    Dance,
    On,
    Off,
    Feedback,
}

impl Mode {
    /// The list of runtime modes that can be cycled through
    const ENABLED_MODES: [Mode; 4] = [
        // Enabled modes
        Mode::Dance,
        Mode::On,
        Mode::Off,
        Mode::Feedback,
    ];

    /// Runs a specific mode, given a reference to the DancingLights construct
    async fn run(&self, lights: &mut DancingLights<'static>) {
        match self {
            Mode::Dance => dance(lights).await,
            Mode::On => on(lights).await,
            Mode::Off => off(lights).await,
            Mode::Feedback => feedback(lights).await,
        }
    }
}

impl defmt::Format for Mode {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Mode::Dance => defmt::write!(fmt, "Dance"),
            Mode::On => defmt::write!(fmt, "On"),
            Mode::Off => defmt::write!(fmt, "Off"),
            Mode::Feedback => defmt::write!(fmt, "Feedback"),
        }
    }
}

/// A helper for rotating through the values in a fixed length array,
/// with wrap around
struct ArrayRotation<'a, T> {
    array: &'a [T],
    next_index: usize,
}

impl<'a, T> ArrayRotation<'a, T> {
    fn new(array: &'a [T]) -> Self {
        ArrayRotation {
            array,
            next_index: 0,
        }
    }

    /// Gets the next value in the array, potentionally wrapping
    /// to the first element if necessary
    fn next(&mut self) -> &T {
        let curr_index = self.next_index;

        self.next_index += 1;
        if self.next_index >= self.array.len() {
            self.next_index = 0;
        }

        &self.array[curr_index]
    }
}

/// Run the supported runtime modes
///
/// Runs the first mode specified in the `ENABLED_MODES` list,
/// cycling to the next mode in order whenever the BOOTSEL button is pressed
pub async fn run(_spawner: Spawner, mut lights: DancingLights<'static>) -> ! {
    let mut rotation = ArrayRotation::new(&Mode::ENABLED_MODES);
    let mut button_reciever = lights
        .bootsel_button
        .receiver()
        .expect("Reciever should be available.");

    loop {
        lights.reset().await;
        select(
            async {
                loop {
                    match (button_reciever).changed().await {
                        ButtonChangeState::Falling => return,
                        ButtonChangeState::Rising => continue,
                    }
                }
            },
            async {
                let mode = rotation.next();
                defmt::info!("Mode: {}", mode);
                mode.run(&mut lights).await;
            },
        )
        .await;
    }
}
