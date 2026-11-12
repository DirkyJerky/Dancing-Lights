use core::future::pending;

use crate::pinout::DancingLights;

pub async fn run(_lights: &mut DancingLights<'static>) -> ! {
    pending::<()>().await;
    unreachable!()
}
