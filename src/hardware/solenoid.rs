use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    peripherals, Peripherals,
};

pub struct Solenoid<'a> {
    pin: Output<'a, AnyPin>,
}

impl<'a> Solenoid<'a> {
    /// Create a new `Solenoid` instance to control whether water is poured via shower or steam wand.
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

    pub fn switch_to_shower(&mut self) {
        self.pin.set_low();
    }

    pub fn switch_to_steam_wand(&mut self) {
        self.pin.set_high();
    }
}
