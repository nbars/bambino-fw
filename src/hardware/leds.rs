//!
//! Everything related to control the portafilter machine LED's.
//!

use embassy_executor::Spawner;
use embassy_futures::select::{self};
use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Speed},
    Peripherals,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

use embassy_stm32::gpio::Pin as _;
use embassy_time::{Duration, Ticker};

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
    /// Cerate a new instance for controlling the LEDs.
    /// # Panics
    /// If there are insufficient ressource for spawning news tasks.
    /// # Safety
    /// This is only safe when called once and without concurrently calling any of the `new()``
    /// methods of the other hardware components.
    pub unsafe fn new(spawner: &mut Spawner) -> Self {
        let p = unsafe { Peripherals::steal() };
        spawner
            .spawn(led_task(LEDKind::OneCup, p.PA15.degrade()))
            .unwrap();
        spawner
            .spawn(led_task(LEDKind::TwoCup, p.PB3.degrade()))
            .unwrap();
        LEDs {}
    }

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
    led: Output<'a, AnyPin>,
}

impl<'a> LEDTask<'a> {
    fn new(led: AnyPin) -> Self {
        let led = Output::new(led, Level::Low, Speed::Low);
        LEDTask { led }
    }

    fn on(&mut self) {
        self.led.set_high();
    }

    fn off(&mut self) {
        self.led.set_low();
    }

    fn toggle(&mut self) {
        self.led.toggle();
    }

    fn is_on(&self) -> bool {
        self.led.is_set_high()
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn led_task(kind: LEDKind, led: AnyPin) -> ! {
    let mut led = LEDTask::new(led);
    led.off();

    let requested_led_state = match kind {
        LEDKind::OneCup => &ONE_CUP_LED_STATE,
        LEDKind::TwoCup => &TWO_CUP_LED_STATE,
    };

    let mut blinking = false;
    let mut ticker = Ticker::every(Duration::from_secs(3600));
    loop {
        match select::select(requested_led_state.wait(), ticker.next()).await {
            select::Either::First(new_state) => match new_state {
                LEDState::On => {
                    ticker = Ticker::every(Duration::from_secs(3600));
                    led.on();
                }
                LEDState::Off => {
                    ticker = Ticker::every(Duration::from_secs(3600));
                    led.off();
                }
                LEDState::Blinking(frequency) => {
                    blinking = true;
                    ticker = Ticker::every(Duration::from_hz(frequency as u64));
                }
            },
            select::Either::Second(_) => {
                if blinking {
                    led.toggle();
                }
            }
        }
    }
}
