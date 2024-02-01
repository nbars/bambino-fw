#![no_std]
#![no_main]

use core::{num::NonZeroU16, time};

use crate::temperature::Temperature;
use buttons::Buttons;
use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select;
use embassy_stm32::adc::Adc;
use embassy_stm32::{
    adc, bind_interrupts,
    exti::{Channel, ExtiInput},
    gpio::{self, Input, Output, Pin, Pull},
    interrupt::typelevel::ADC1,
    peripherals::ADC,
    Peripheral,
};
use embassy_time::{Delay, Duration, Timer};
use heater::Heater;
use leds::LEDs;
use pump::Pump;
use solenoid::Solenoid;

use {defmt_rtt as _, panic_probe as _};

mod buttons;
mod flow_meter;
mod heater;
mod leds;
mod pump;
mod solenoid;
mod temperature;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let mut solenoid = Solenoid::new(p.PA11.degrade());

    let mut buttons = Buttons::new(
        p.PA0.degrade(),
        p.EXTI0.degrade(),
        p.PA1.degrade(),
        p.EXTI1.degrade(),
        p.PA2.degrade(),
        p.EXTI2.degrade(),
        p.PA3.degrade(),
        p.EXTI3.degrade(),
    );

    let mut leds = LEDs::new(p.PA15.degrade(), p.PB3.degrade());

    let mut temperatur = Temperature::new(p.ADC, p.PB1);

    let mut pump = Pump::new(p.PB8, p.TIM16);

    let mut heater = Heater::new(p.PB6.degrade());

    // let flow_signal = p.PA7.degrade();
    // let flow_enable = p.PB11.degrade();

    // let radiator = p.PB6.degrade();

    solenoid.switch_to_steam_wand();

    loop {
        let button = buttons.wait_for_button_press(Some(Duration::from_millis(50)));
        let timer = Timer::after_millis(200);

        let event = select::select(button, timer).await;

        match event {
            select::Either::First(event) => {
                let button = event.source();
                match button {
                    buttons::ButtonKind::OneCup => pump.set_power(pump::PumpPower::Lowest),
                    buttons::ButtonKind::TwoCup => pump.set_power(pump::PumpPower::Highest),
                    buttons::ButtonKind::HotWater => {
                        if heater.is_on() {
                            heater.off()
                        } else {
                            heater.on();
                        }
                    }
                    buttons::ButtonKind::Steam => pump.toggle(),
                }
            }
            select::Either::Second(_) => {
                let t = temperatur
                    .read_averaged_celcius(NonZeroU16::new(1000).unwrap())
                    .await;
                info!("t: {}°C", t);
            }
        }
    }

    // create state machine. Default is power-save that wait for external inputs.
    // Based on input, create a new state machine and execute it.
}
