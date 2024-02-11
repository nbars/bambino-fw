//!
//! Everything related to control the pump of the coffeemachine.
//!

use embassy_stm32::{
    gpio::OutputType,
    peripherals::TIM16,
    time::Hertz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Channel, CountingMode,
    },
    Peripheral, Peripherals,
};

const SPEED_LOWER_BOUND: u16 = 5;

/// The power level of the pump.
pub enum PumpPower {
    /// Set pump to the lowest powerlevel that still allows to move the water.
    Lowest,
    /// Set the pump to the highest power level.
    Highest,
    /// Specific fraction of the maximal speed. This must be <= 1.
    /// Setting this to 0.0 is equivalent to `Lowest` and 1.0 is the same as Highest.
    Fraction(f32),
}

/// The water pump of the machine.
pub struct Pump<'a, T> {
    pwm: SimplePwm<'a, T>,
}

impl<'a> Pump<'a, TIM16> {
    /// Create a new `Pump` instance.
    pub fn new(p: &Peripherals) -> Self {
        let pin = PwmPin::new_ch1(unsafe { p.PB8.clone_unchecked() }, OutputType::PushPull);
        let pwm = SimplePwm::new(
            unsafe { p.TIM16.clone_unchecked() },
            Some(pin),
            None,
            None,
            None,
            Hertz::hz(16),
            CountingMode::EdgeAlignedUp,
        );

        // This is the hack to set the MOE bit for our timer, since this
        // is currently buggy in embassy.
        const TIM16: embassy_stm32::pac::timer::TimAdv =
            unsafe { embassy_stm32::pac::timer::TimAdv::from_ptr(0x4001_4400_usize as _) };
        TIM16.bdtr().modify(|r| r.set_moe(true));

        let mut ret = Pump { pwm };
        ret.set_power(PumpPower::Highest);
        ret
    }

    /// The the power of the pump to `power`.
    pub fn set_power(&mut self, power: PumpPower) {
        let max_duty = self.pwm.get_max_duty();
        debug_assert!(max_duty > SPEED_LOWER_BOUND);
        match power {
            PumpPower::Lowest => self.pwm.set_duty(Channel::Ch1, SPEED_LOWER_BOUND),
            PumpPower::Highest => self.pwm.set_duty(Channel::Ch1, max_duty),
            PumpPower::Fraction(frac) => {
                assert!(frac <= 1.0);
                self.pwm.set_duty(
                    Channel::Ch1,
                    (frac * (max_duty as f32 - SPEED_LOWER_BOUND as f32)) as u16,
                );
            }
        }
    }

    /// Turn the pump on if it is off, and vice versa.
    pub fn toggle(&mut self) {
        if self.pwm.is_enabled(Channel::Ch1) {
            self.disable()
        } else {
            self.enable()
        }
    }

    /// Turn the pump on.
    pub fn enable(&mut self) {
        self.pwm.enable(Channel::Ch1);
    }

    /// Turn the pump off.
    pub fn disable(&mut self) {
        self.pwm.disable(Channel::Ch1);
    }

    /// Get the maximum raw power value the is allowed to be passed to `set_raw_power()`.
    pub fn get_max_raw_power_value(&self) -> u16 {
        self.pwm.get_max_duty()
    }

    /// Set the raw power of the pump to `power`.
    ///
    /// # Panics
    /// If the passed `power` value is greater than `Self::get_max_raw_power_value`.
    pub fn set_raw_power(&mut self, power: u16) {
        let max_duty = self.pwm.get_max_duty();
        assert!(power <= max_duty);
        self.pwm.set_duty(Channel::Ch1, power);
    }
}
