use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};

pub struct LEDs<'a> {
    one_cup: Output<'a, AnyPin>,
    two_cup: Output<'a, AnyPin>,
}

impl<'a> LEDs<'a> {
    pub fn new(one_cup: AnyPin, two_cup: AnyPin) -> Self {
        let one_cup = Output::new(one_cup, Level::Low, Speed::Low);
        let two_cup = Output::new(two_cup, Level::Low, Speed::Low);
        LEDs { one_cup, two_cup }
    }

    pub fn off(&mut self) {
        self.set_one_cup(false);
        self.set_two_cup(false);
    }

    pub fn set_one_cup(&mut self, on: bool) {
        if on {
            self.one_cup.set_high();
        } else {
            self.one_cup.set_low();
        }
    }

    pub fn set_two_cup(&mut self, on: bool) {
        if on {
            self.two_cup.set_high();
        } else {
            self.two_cup.set_low();
        }
    }

    pub fn toggle_one_cup(&mut self) {
        self.one_cup.toggle();
    }

    pub fn toggle_two_cup(&mut self) {
        self.two_cup.toggle();
    }
}
