//!
//! Everything related to the button of the machine.
//!
use embassy_executor::Spawner;
use embassy_futures::select::{self};
use embassy_stm32::{
    exti::{Channel as _, ExtiInput},
    gpio::{self, AnyPin, Pin},
    Peripheral, Peripherals,
};

use futures::FutureExt;

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::{Duration, Instant};

#[derive(defmt::Format, Clone, Copy)]
/// The kind of button that may be pressed or released.
pub enum ButtonKind {
    /// The one cup button.
    OneCup,
    /// The two cups button.
    TwoCup,
    /// The hot water button.
    HotWater,
    /// The steam button.
    Steam,
}

/// Event emitted if a button changes its state from pressed to released or
/// the other way around.
#[derive(Clone, Copy, defmt::Format)]
pub struct ButtonEvent {
    /// The button that was pressed.
    source: ButtonKind,
    /// The new state of the button.
    state: ButtonState,
    /// The time the event was registered.
    timestamp: Instant,
}

impl ButtonEvent {
    /// Create a new button event.
    pub fn new(source: ButtonKind, state: ButtonState, timestamp: Instant) -> Self {
        ButtonEvent {
            source,
            state,
            timestamp,
        }
    }

    /// Construct a new `ButtonEvent` with inverted button state and timestamp
    /// set to `Instant::now()`.
    fn state_transition(self) -> ButtonEvent {
        ButtonEvent::new(self.source, self.state.not(), Instant::now())
    }

    /// The time elapsed since recording this event.
    pub fn elapsed(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// The button that caused the event.
    pub fn source(&self) -> ButtonKind {
        self.source
    }

    /// The new state of the button.
    pub fn state(&self) -> ButtonState {
        self.state
    }

    /// The timestamp at which this event happend.
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }
}

/// State of a button. Ether `Pressed` or `Released`.
#[derive(Clone, Copy, defmt::Format)]
pub enum ButtonState {
    /// The button is pressed.
    Pressed,
    /// The button is released.
    Released,
}

impl ButtonState {
    fn not(&self) -> ButtonState {
        match self {
            ButtonState::Pressed => ButtonState::Released,
            ButtonState::Released => ButtonState::Pressed,
        }
    }
}

/// Event emmited if a button changes from pressed to released or the other way around.
#[derive(defmt::Format)]
pub struct ButtonStateTransitionEvent {
    old_state: ButtonEvent,
    new_state: ButtonEvent,
}

impl ButtonStateTransitionEvent {
    /// Get the old state of the button before the transition happend.
    /// This is especially useful in order to compute the duration of, e.g.,
    /// a button press.
    pub fn old_state(&self) -> &ButtonEvent {
        &self.old_state
    }

    /// The new (current) state of the button.
    pub fn new_state(&self) -> &ButtonEvent {
        &self.new_state
    }
}

type ButtonEventChannel =
    embassy_sync::channel::Channel<ThreadModeRawMutex, ButtonStateTransitionEvent, 6>;
static BUTTON_EVENT_CHANNEL: ButtonEventChannel = ButtonEventChannel::new();
const DEBOUNCE_INTERVAL: Duration = Duration::from_millis(50);

/// All buttons of the machine.
pub struct Buttons;

impl Buttons {
    /// Create a new `Buttons` instance.
    ///
    /// # Panics
    /// If the spawner fails to spawn the task.
    pub fn new(spawner: &mut Spawner) -> Self {
        spawner.spawn(button_task()).unwrap();
        Buttons {}
    }

    /// Wait for any button to change its state from pressed to released or released
    /// to pressed. Button pressen are queued up, such that this function must not
    /// be awaited the whole time. However, the queue buffer is limited, such that
    /// this function should be called preiodically in order to avoid state transitions
    /// to be lost.
    pub async fn wait_for_button_state_change(&mut self) -> ButtonStateTransitionEvent {
        BUTTON_EVENT_CHANNEL.receive().await
    }
}

struct ButtonsTask<'a> {
    one_cup_exti: ExtiInput<'a, AnyPin>,
    two_cup_exti: ExtiInput<'a, AnyPin>,
    steam_exti: ExtiInput<'a, AnyPin>,
    hot_water_exti: ExtiInput<'a, AnyPin>,
    last_one_cup_event: ButtonEvent,
    last_two_cup_event: ButtonEvent,
    last_hot_water_event: ButtonEvent,
    last_steam_event: ButtonEvent,
}

impl<'a> ButtonsTask<'a> {
    fn new(p: Peripherals) -> Self {
        let one_cup_input = gpio::Input::new(
            unsafe { p.PA0.clone_unchecked().degrade() },
            gpio::Pull::None,
        );
        let one_cup_exti = ExtiInput::new(one_cup_input, unsafe {
            p.EXTI0.clone_unchecked().degrade()
        });

        let two_cup_input = gpio::Input::new(
            unsafe { p.PA1.clone_unchecked().degrade() },
            gpio::Pull::None,
        );
        let two_cup_exti = ExtiInput::new(two_cup_input, unsafe {
            p.EXTI1.clone_unchecked().degrade()
        });

        let steam_input = gpio::Input::new(
            unsafe { p.PA2.clone_unchecked().degrade() },
            gpio::Pull::None,
        );
        let steam_exti =
            ExtiInput::new(steam_input, unsafe { p.EXTI2.clone_unchecked().degrade() });

        let hot_water_input = gpio::Input::new(
            unsafe { p.PA3.clone_unchecked().degrade() },
            gpio::Pull::None,
        );
        let hot_water_exti = ExtiInput::new(hot_water_input, unsafe {
            p.EXTI3.clone_unchecked().degrade()
        });

        let last_one_cup_event =
            ButtonEvent::new(ButtonKind::OneCup, ButtonState::Released, Instant::now());
        let last_two_cup_event =
            ButtonEvent::new(ButtonKind::TwoCup, ButtonState::Released, Instant::now());
        let last_hot_water_event =
            ButtonEvent::new(ButtonKind::HotWater, ButtonState::Released, Instant::now());
        let last_steam_event =
            ButtonEvent::new(ButtonKind::Steam, ButtonState::Released, Instant::now());

        ButtonsTask {
            one_cup_exti,
            two_cup_exti,
            hot_water_exti,
            steam_exti,
            last_one_cup_event,
            last_two_cup_event,
            last_hot_water_event,
            last_steam_event,
        }
    }

    fn kind_to_last_event(&self, kind: ButtonKind) -> &ButtonEvent {
        match kind {
            ButtonKind::OneCup => &self.last_one_cup_event,
            ButtonKind::TwoCup => &self.last_two_cup_event,
            ButtonKind::HotWater => &self.last_hot_water_event,
            ButtonKind::Steam => &self.last_steam_event,
        }
    }

    fn kind_to_last_event_mut(&mut self, kind: ButtonKind) -> &mut ButtonEvent {
        match kind {
            ButtonKind::OneCup => &mut self.last_one_cup_event,
            ButtonKind::TwoCup => &mut self.last_two_cup_event,
            ButtonKind::HotWater => &mut self.last_hot_water_event,
            ButtonKind::Steam => &mut self.last_steam_event,
        }
    }

    async fn wait_for_button_event(&mut self) -> ButtonEvent {
        let one_cup_watch = match self.last_one_cup_event.state {
            ButtonState::Pressed => self.one_cup_exti.wait_for_low().left_future(),
            ButtonState::Released => self.one_cup_exti.wait_for_high().right_future(),
        };
        let two_cup_watch = match self.last_two_cup_event.state {
            ButtonState::Pressed => self.two_cup_exti.wait_for_low().left_future(),
            ButtonState::Released => self.two_cup_exti.wait_for_high().right_future(),
        };
        let hot_water_watch = match self.last_hot_water_event.state {
            ButtonState::Pressed => self.hot_water_exti.wait_for_low().left_future(),
            ButtonState::Released => self.hot_water_exti.wait_for_high().right_future(),
        };
        let steam_watch = match self.last_steam_event.state {
            ButtonState::Pressed => self.steam_exti.wait_for_low().left_future(),
            ButtonState::Released => self.steam_exti.wait_for_high().right_future(),
        };

        match select::select4(one_cup_watch, two_cup_watch, hot_water_watch, steam_watch).await {
            select::Either4::First(_) => self.last_one_cup_event.state_transition(),
            select::Either4::Second(_) => self.last_two_cup_event.state_transition(),
            select::Either4::Third(_) => self.last_hot_water_event.state_transition(),
            select::Either4::Fourth(_) => self.last_steam_event.state_transition(),
        }
    }

    async fn wait_for_button_event_debounced(&mut self) -> (ButtonEvent, ButtonEvent) {
        loop {
            let event @ ButtonEvent { source, .. } = self.wait_for_button_event().await;
            {
                let elapsed_since_last_event = self.kind_to_last_event(source).elapsed();
                if elapsed_since_last_event < DEBOUNCE_INTERVAL {
                    continue;
                }
                let old_event = self.kind_to_last_event_mut(source);
                let old_event_copy = *old_event;
                *old_event = event;
                return (old_event_copy, event);
            }
        }
    }
}

#[embassy_executor::task]
async fn button_task() -> ! {
    let p = unsafe { Peripherals::steal() };
    let mut buttons = ButtonsTask::new(p);

    loop {
        let (old_state, new_state) = buttons.wait_for_button_event_debounced().await;
        let event = ButtonStateTransitionEvent {
            old_state,
            new_state,
        };

        BUTTON_EVENT_CHANNEL.send(event).await;
    }
}
