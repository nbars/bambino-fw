//!
//! Everything related to control the portafilter machine LED's.
//!

use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Speed},
    Peripherals,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

use embassy_stm32::gpio::Pin as _;

static ONE_CUP_LED_STATE: Signal<ThreadModeRawMutex, LEDState> = Signal::new();
static TWO_CUP_LED_STATE: Signal<ThreadModeRawMutex, LEDState> = Signal::new();

type Hz = u8;

/// The state if an LED.
#[derive(Clone, Copy)]
pub enum LEDState {
    /// LED is on.
    On,
    /// LED is off.
    Off,
    /// LED is blinking with the given frequency.
    Blinking(Hz),
}

/// All the controllable LEDs.
#[derive(Clone, Copy)]
pub enum LEDKind {
    /// The one cup button's LED.
    OneCup,
    /// The two cups button's LED.
    TwoCup,
}

/// All controllable LEDs of the machine.
pub struct LEDs;

impl LEDs {
    /// Turn all LEDs off.
    pub fn off(&mut self) {
        self.set_state_all(LEDState::Off);
    }

    /// Set all LEDs to `new_state`.
    pub fn set_state_all(&mut self, new_state: LEDState) {
        ONE_CUP_LED_STATE.signal(new_state);
        TWO_CUP_LED_STATE.signal(new_state);
    }

    /// Set `led` to `new_state`.
    pub fn set_state(&mut self, led: LEDKind, new_state: LEDState) {
        match led {
            LEDKind::OneCup => ONE_CUP_LED_STATE.signal(new_state),
            LEDKind::TwoCup => TWO_CUP_LED_STATE.signal(new_state),
        }
    }
}

struct LEDTask<'a> {
    one_cup: Output<'a, AnyPin>,
    two_cup: Output<'a, AnyPin>,
}

impl<'a> LEDTask<'a> {
    fn new(p: Peripherals) -> Self {
        let one_cup = Output::new(p.PA15.degrade(), Level::Low, Speed::Low);
        let two_cup = Output::new(p.PB3.degrade(), Level::Low, Speed::Low);
        LEDTask { one_cup, two_cup }
    }

    fn off(&mut self) {
        self.set_one_cup(false);
        self.set_two_cup(false);
    }

    fn set_one_cup(&mut self, on: bool) {
        if on {
            self.one_cup.set_high();
        } else {
            self.one_cup.set_low();
        }
    }

    fn set_two_cup(&mut self, on: bool) {
        if on {
            self.two_cup.set_high();
        } else {
            self.two_cup.set_low();
        }
    }

    fn toggle_one_cup(&mut self) {
        self.one_cup.toggle();
    }

    fn toggle_two_cup(&mut self) {
        self.two_cup.toggle();
    }
}

#[embassy_executor::task]
async fn led_task() -> ! {
    let p = unsafe { Peripherals::steal() };
    let mut leds = LEDTask::new(p);

    loop {}
}
