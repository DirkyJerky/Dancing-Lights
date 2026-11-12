use embassy_executor::Spawner;

use crate::pinout::DancingLights;

use super::dance::run as dance;

pub async fn run(spawner: Spawner, lights: DancingLights<'static>) -> ! {
    dance(spawner, lights).await
}
