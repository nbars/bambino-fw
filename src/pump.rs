use defmt::info;
use embassy_stm32::{
    gpio::{AnyPin, OutputType},
    time::Hertz,
    timer::{
        simple_pwm::{Ch1, PwmPin, SimplePwm},
        Channel, Channel1Pin, CountingMode,
    },
    Peripheral,
};
use embassy_time::Timer;

const SPEED_LOWER_BOUND: u16 = 6250;

pub enum PumpPower {
    /// Set pump to the lowest powerlevel that still allows to move the water.
    Lowest,
    /// Set the pump to the highest power level.
    Highest,
    /// Specific fraction of the maximal speed. This must be <= 1.
    /// Setting this to 0.0 is equivalent to `Lowest` and 1.0 is the same as Highest.
    Fraction(f32),
}

pub struct Pump<'a, T> {
    pwm: SimplePwm<'a, T>,
}

impl<'a, T> Pump<'a, T>
where
    T: embassy_stm32::timer::CaptureCompare16bitInstance,
{
    pub fn new<Ti>(pin: impl Channel1Pin<T>, timer: Ti) -> Self
    where
        Ti: Peripheral<P = T> + 'a,
    {
        let pin = PwmPin::new_ch1(pin, OutputType::PushPull);
        let mut pwm = SimplePwm::new(
            timer,
            Some(pin),
            None,
            None,
            None,
            Hertz::hz(64),
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

    pub fn toggle(&mut self) {
        if self.pwm.is_enabled(Channel::Ch1) {
            self.disable()
        } else {
            self.enable()
        }
    }

    pub fn enable(&mut self) {
        self.pwm.enable(Channel::Ch1);
    }

    pub fn disable(&mut self) {
        self.pwm.disable(Channel::Ch1);
    }
}
