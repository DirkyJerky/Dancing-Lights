use cyw43::Control;

pub struct Cyw43_GPIO {
    control: Control<'static>,
}

impl Cyw43_GPIO {
    pub fn new(control: Control<'static>) -> Self {
        Self { control }
    }

    pub async fn up(&mut self, pin: u8) {
        if pin < 3 {
            self.control.gpio_set(pin, true).await;
        }
    }

    pub async fn down(&mut self, pin: u8) {
        if pin < 3 {
            self.control.gpio_set(pin, false).await;
        }
    }

    pub async fn reset(&mut self) {
        self.down(0).await;
        self.down(1).await;
        self.down(2).await;
    }
}
