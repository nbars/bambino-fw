use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    Peripherals,
};

pub struct Heater<'a> {
    pin: Output<'a, AnyPin>,
}

impl<'a> Heater<'a> {
    pub fn new(p: &Peripherals) -> Self {
        let pin = Output::new(p.PB6.degrade(), Level::Low, Speed::Low);
        Heater { pin }
    }

    pub fn on(&mut self) {
        self.pin.set_high()
    }

    pub fn off(&mut self) {
        self.pin.set_low()
    }

    pub fn is_on(&self) -> bool {
        self.pin.is_set_high()
    }
}
