#![no_std]
#![no_main]

// use core::num::NonZeroU16;

use bambino_fw::hardware::{buttons, pump};
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use defmt::*;

#[embassy_executor::main]
async fn main(mut spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut pump = unsafe { pump::Pump::new() };
    let mut buttons = unsafe { buttons::Buttons::new(&mut spawner) };

    pump.set_power(pump::PumpPower::Lowest);
    pump.enable();

    Timer::after_millis(5000).await;

    pump.set_power(pump::PumpPower::Highest);

    Timer::after_millis(3000).await;
    pump.disable();

    loop {
        Timer::after_millis(200).await;
    }
}
