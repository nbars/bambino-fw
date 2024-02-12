//!
//! Module to control the heater.
//!

use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    Peripherals,
};

/// The heater used to boil the water.
pub struct Heater<'a> {
    pin: Output<'a, AnyPin>,
}

impl<'a> Heater<'a> {
    /// Create a new heater instance.
    pub fn new(p: Peripherals) -> Self {
        let pin = Output::new(p.PB6.degrade(), Level::Low, Speed::Low);
        Heater { pin }
    }

    /// Turn the heater on.
    pub fn on(&mut self) {
        self.pin.set_high();
    }

    /// Turn the heater off.
    pub fn off(&mut self) {
        self.pin.set_low();
    }

    /// Check whether the heater is currently on.
    pub fn is_on(&self) -> bool {
        self.pin.is_set_high()
    }
}

impl<'a> Drop for Heater<'a> {
    fn drop(&mut self) {
        self.off();
    }
}
