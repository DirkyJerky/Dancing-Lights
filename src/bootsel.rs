use cyw43::Control;
use embassy_rp::{
    Peri, bootsel::is_bootsel_pressed, peripherals::BOOTSEL, spinlock_mutex::SpinlockRawMutex,
};
use embassy_sync::watch::{Receiver, Sender, Watch};
use embassy_time::{Duration, Timer};

static SPINLOCK_ID: usize = 0;
static WATCHER_COUNT: usize = 4;

type MUTEX = SpinlockRawMutex<SPINLOCK_ID>;

static WATCH: Watch<MUTEX, ButtonChangeState, WATCHER_COUNT> = Watch::new();

#[derive(Debug, Clone, Copy)]
pub enum ButtonChangeState {
    Rising,
    Falling,
}

pub struct BootselButton;

pub struct BootselPoller<'a> {
    button: Peri<'static, BOOTSEL>,
    sender: Sender<'a, MUTEX, ButtonChangeState, WATCHER_COUNT>,
    poll_frequency: Duration,
    last_value: bool,
}

impl<'a> BootselPoller<'a> {
    fn new(
        mut button: Peri<'static, BOOTSEL>,
        sender: Sender<'a, MUTEX, ButtonChangeState, WATCHER_COUNT>,
        poll_frequency: Duration,
    ) -> Self {
        let current_state = is_bootsel_pressed(button.reborrow());
        Self {
            button,
            sender,
            poll_frequency,
            last_value: current_state,
        }
    }

    pub async fn start_polling(mut self) -> ! {
        loop {
            let current_state = is_bootsel_pressed(self.button.reborrow());

            if let Some(change_state) = match (self.last_value, current_state) {
                (false, true) => Some(ButtonChangeState::Rising),
                (true, false) => Some(ButtonChangeState::Falling),
                (_, _) => None,
            } {
                self.sender.send(change_state);
            }

            self.last_value = current_state;

            Timer::after(self.poll_frequency).await;
        }
    }
}

impl BootselButton {
    pub fn new(button: Peri<'static, BOOTSEL>, poll_frequency: Duration) -> (Self, BootselPoller) {
        let poller: BootselPoller<'static> =
            BootselPoller::new(button, WATCH.sender(), poll_frequency);

        (Self {}, poller)
    }

    pub fn receiver(
        &mut self,
    ) -> Option<Receiver<'static, MUTEX, ButtonChangeState, WATCHER_COUNT>> {
        WATCH.receiver()
    }
}
