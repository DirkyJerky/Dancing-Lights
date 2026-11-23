# Firmware architecture

The firmware uses Rust and Embassy for asynchronous hardware control on the Raspberry Pi Pico W.

- **Peripheral setup:** `main.rs` initializes hardware, configures PWM and communication interfaces, and starts persistent CYW43 and BOOTSEL background tasks.
- **Device construction:** `pinout.rs` groups hardware resources into device drivers and consolidates them into `DancingLights`, which owns the application's hardware.
- **Runtime modes:** `run/entry.rs` rotates through configured modes, including Dance, On, Off, and Feedback. Each mode borrows the existing devices; a BOOTSEL button release cancels the current mode, resets the lighting outputs, and starts the next.
- **Shared utilities:** `utils/` provides PWM brightness control, gamma correction, trigonometric approximations, PID control, and LED-array gauges.
