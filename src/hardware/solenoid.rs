//!
//! This module contains the implementation for the solenoid, which is an actor
//! that allows to switch the water flow to be poured via steam wand or shower.
//!

use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    Peripherals,
};

/// The ways water can be dispensed.
pub enum WaterOutputKind {
    /// via the shower head
    Shower,
    /// via the steam wand
    SteamWand,
}

/// Device to control whether water is flowing through the steam wand,
/// or through the show.
pub struct Solenoid<'a> {
    pin: Output<'a, AnyPin>,
}

impl<'a> Solenoid<'a> {
    /// Create a new `Solenoid` instance to control whether water is poured via shower or steam wand.
    /// By default, the the water is poured via the shower.
    /// # Safety
    /// This is only safe when called once and without concurrently calling any of the `new()``
    /// methods of the other hardware components.
    pub unsafe fn new() -> Self {
        let p = unsafe { Peripherals::steal() };
        let pin = p.PA11.degrade();
        Solenoid {
            pin: Output::new(pin, Level::Low, Speed::Low),
        }
    }

    /// Switch the water to be dispensed via `output`.
    pub fn switch(&mut self, output: WaterOutputKind) {
        match output {
            WaterOutputKind::Shower => self.switch_to_shower(),
            WaterOutputKind::SteamWand => self.switch_to_steam_wand(),
        }
    }

    /// Dispense the water via the shower. This is the default mode.
    pub fn switch_to_shower(&mut self) {
        self.pin.set_low();
    }

    /// Dispense the water (steam) via the steam wand.
    pub fn switch_to_steam_wand(&mut self) {
        self.pin.set_high();
    }
}
