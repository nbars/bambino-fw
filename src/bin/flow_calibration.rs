#![no_std]
#![no_main]

use bambino_fw::hardware::{buttons, leds};
use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(mut spawner: Spawner) -> ! {
    let _p = embassy_stm32::init(Default::default());
    //let mut pump = pump::Pump::new(&p);

    let mut buttons = buttons::Buttons::new(&mut spawner);
    let mut leds = leds::LEDs::new(&mut spawner);
    leds.set_state_all(leds::LEDState::On);

    /*
    Calibrate the following things:
        - the ml/pulse
     */

    // pump.set_power(pump::PumpPower::Lowest);
    // pump.enable();

    // Timer::after_millis(5000).await;

    // pump.set_power(pump::PumpPower::Highest);

    // Timer::after_millis(3000).await;
    // pump.disable();

    loop {
        let event = buttons.wait_for_button_state_change().await;
        let new_state = event.new_state().state();
        let source = event.new_state().source();
        info!("Button {:?} is now in state {:?}", &source, new_state);

        match source {
            buttons::ButtonKind::OneCup => {
                leds.set_state(leds::LEDKind::OneCup, leds::LEDState::Blinking(2))
            }
            buttons::ButtonKind::TwoCup => {
                leds.set_state(leds::LEDKind::TwoCup, leds::LEDState::Blinking(3))
            }
            _ => (),
        }
    }
}
