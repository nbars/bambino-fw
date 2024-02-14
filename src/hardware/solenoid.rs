use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    peripherals, Peripherals,
};

pub struct Solenoid<'a> {
    pin: Output<'a, AnyPin>,
}

impl<'a> Solenoid<'a> {
    pub fn new(p: &Peripherals) -> Self {
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
