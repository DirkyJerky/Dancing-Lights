# Dancing Lights

A Pi Pico, some LEDs, and a dream!

This firmware configures a Raspberry Pi Pico W to control colored LEDs and other peripherals, using different behavior modes that are selected at runtime.  This firmware is created with [Embassy](https://embassy.dev/), and flashed using probe-rs with a SWD probe.

## Setup

<img src="pinout.jpg" alt="Breadboard setup with a Raspberry Pi Pico W, colored LED arrays, and a photoresistor" width="700">

The breadboard centers on a Raspberry Pi Pico W connected to three groups of colored LEDs, an RGB LED, a photoresistor, and a separate LED on jumper wires. Resistors limit LED current and form the photoresistor's sensing circuit using a voltage divider, while a breadboard power module supplies the power rails.

The colored LED groups display lighting effects, including randomized effects and PID feedback guages. The separate LED can be moved closer to or farther from the photoresistor to experiment with automatic brightness control.
