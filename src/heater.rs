use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Speed},
    timer::{simple_pwm::SimplePwm, Channel1Pin, Channel2Pin, Channel3Pin, Channel4Pin},
    Peripheral,
};

pub struct Heater<'a> {
    pin: Output<'a, AnyPin>,
}

impl<'a> Heater<'a> {
    pub fn new(pin: AnyPin) -> Self {
        let pin = Output::new(pin, Level::Low, Speed::Low);
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
