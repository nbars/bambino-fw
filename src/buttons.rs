//!
//! Module the get the status of all physical button of the machine.
//!

use embassy_futures::select::{self, select4};
use embassy_stm32::{
    exti::{AnyChannel, ExtiInput},
    gpio::{self, AnyPin},
    Peripheral,
};

use embassy_time::{Duration, Instant, Timer};

#[derive(Clone, Copy)]
pub enum ButtonKind {
    OneCup,
    TwoCup,
    HotWater,
    Steam,
}

pub struct ButtonPressEvent {
    ts: Instant,
    button: ButtonKind,
}

impl ButtonPressEvent {
    pub fn start_ts(&self) -> Instant {
        self.ts
    }

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

    // pub fn new(
    //     one_cup: ExtiInput<'a, AnyPin>,
    //     two_cup: ExtiInput<'a, AnyPin>,
    //     hot_water: ExtiInput<'a, AnyPin>,
    //     steam: ExtiInput<'a, AnyPin>,
    // ) -> Self {
    //     Buttons {
    //         one_cup,
    //         two_cup,
    //         hot_water,
    //         steam,
    //     }
    // }

    pub async fn wait_for_button_press(&mut self, debounce: Option<Duration>) -> ButtonPressEvent {
        if let (Some(last_poll), Some(debounce)) = (self.last_poll, debounce) {
            let diff = last_poll.elapsed() - debounce;
            self.last_poll = Some(Instant::now());
            if diff.as_millis() > 0 {
                Timer::after(diff).await;
            }
        }

        let selection = select4(
            self.one_cup_exti.wait_for_high(),
            self.two_cup_exti.wait_for_high(),
            self.hot_water_exti.wait_for_high(),
            self.steam_exti.wait_for_high(),
        )
        .await;
        let ts = Instant::now();
        let button = match selection {
            select::Either4::First(_) => ButtonKind::OneCup,
            select::Either4::Second(_) => ButtonKind::TwoCup,
            select::Either4::Third(_) => ButtonKind::HotWater,
            select::Either4::Fourth(_) => ButtonKind::Steam,
        };
        ButtonPressEvent { ts, button }
    }

}
