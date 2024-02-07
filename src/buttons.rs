//!
//! Module the get the status of all physical buttons of the machine.
//!

use embassy_futures::select::{self, select4};
use embassy_stm32::{
    exti::{AnyChannel, ExtiInput},
    gpio::{self, AnyPin},
    Peripheral,
};

use embassy_time::{Duration, Instant, Timer};

#[derive(Clone, Copy)]
/// The button that caused the event.
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

/// Event that is fired if a button was pressed.
pub struct ButtonPressEvent {
    /// The time the button press was registered.
    ts: Instant,
    /// The button that was pressed.
    button: ButtonKind,
}

impl ButtonPressEvent {
    /// The time the button was pressed.
    pub fn timestamp(&self) -> Instant {
        self.ts
    }

    /// The orginating button of the event.
    pub fn source(&self) -> ButtonKind {
        self.button
    }
}

pub struct Buttons<'a> {
    one_cup_exti: ExtiInput<'a, AnyPin>,
    two_cup_exti: ExtiInput<'a, AnyPin>,
    steam_exti: ExtiInput<'a, AnyPin>,
    hot_water_exti: ExtiInput<'a, AnyPin>,
    last_poll: Option<Instant>,
}

impl<'a> Buttons<'a> {
    pub fn new<OneCupPin, TwoCupPin, SteamPin, HotWaterPin>(
        one_cup_pin: OneCupPin,
        one_cup_pin_exti: impl Peripheral<P = OneCupPin::ExtiChannel> + 'a,
        two_cup_pin: TwoCupPin,
        two_cup_pin_exti: impl Peripheral<P = TwoCupPin::ExtiChannel> + 'a,
        steam_pin: SteamPin,
        steam_pin_exti: impl Peripheral<P = SteamPin::ExtiChannel> + 'a,
        hot_water_pin: HotWaterPin,
        hot_water_pin_exti: impl Peripheral<P = HotWaterPin::ExtiChannel> + 'a,
    ) -> Self
    where
        OneCupPin: gpio::Pin<ExtiChannel = AnyChannel>,
        TwoCupPin: gpio::Pin<ExtiChannel = AnyChannel>,
        HotWaterPin: gpio::Pin<ExtiChannel = AnyChannel>,
        SteamPin: gpio::Pin<ExtiChannel = AnyChannel>,
    {
        let one_cup_input = gpio::Input::new(one_cup_pin.degrade(), gpio::Pull::None);
        let one_cup_exti = ExtiInput::new(one_cup_input, one_cup_pin_exti);

        let two_cup_input = gpio::Input::new(two_cup_pin.degrade(), gpio::Pull::None);
        let two_cup_exti = ExtiInput::new(two_cup_input, two_cup_pin_exti);

        let steam_input = gpio::Input::new(steam_pin.degrade(), gpio::Pull::None);
        let steam_exti = ExtiInput::new(steam_input, steam_pin_exti);

        let hot_water_input = gpio::Input::new(hot_water_pin.degrade(), gpio::Pull::None);
        let hot_water_exti = ExtiInput::new(hot_water_input, hot_water_pin_exti);

        Buttons {
            one_cup_exti,
            two_cup_exti,
            hot_water_exti,
            steam_exti,
            last_poll: None,
        }
    }

    pub async fn wait_for_button_press(&mut self, debounce: Option<Duration>) -> ButtonPressEvent {
        if let (Some(last_poll), Some(debounce)) = (self.last_poll, debounce) {
            let diff = last_poll.elapsed();
            if diff < debounce {
                Timer::after(debounce - diff).await;
            }
        }

        let selection = select4(
            self.one_cup_exti.wait_for_rising_edge(),
            self.two_cup_exti.wait_for_rising_edge(),
            self.hot_water_exti.wait_for_rising_edge(),
            self.steam_exti.wait_for_rising_edge(),
        )
        .await;

        let ts = Instant::now();
        self.last_poll = Some(ts);
        let button = match selection {
            select::Either4::First(_) => ButtonKind::OneCup,
            select::Either4::Second(_) => ButtonKind::TwoCup,
            select::Either4::Third(_) => ButtonKind::HotWater,
            select::Either4::Fourth(_) => ButtonKind::Steam,
        };
        ButtonPressEvent { ts, button }
    }
}
