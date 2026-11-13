use cyw43::Control;

pub struct Cyw43_GPIO {
    control: Control<'static>,
}

impl Cyw43_GPIO {
    pub fn new(control: Control<'static>) -> Self {
        Self { control }
    }

    pub async fn up(&mut self, pin: u8) {
        self.control.gpio_set(pin, true).await;
    }

    pub async fn down(&mut self, pin: u8) {
        self.control.gpio_set(pin, false).await;
    }

    pub async fn reset(&mut self) {
        for i in 0..8 {
            self.down(i).await;
        }
    }
}
