#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
use bambino_fw::{buttons, pump};
use embassy_executor::Spawner;

#[allow(unused)]
use defmt::*;

#[embassy_executor::main]
async fn main(mut spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut pump = pump::Pump::new(&p);
    let mut buttons = buttons::Buttons::new(&mut spawner);

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
        info!("Button {:?} is now in state {:?}", source, new_state);
    }
}
