#![no_std]
#![no_main]

use bambino_fw::{hardware::{
    buttons::{self, ButtonState}, flow_meter::{self, FlowMeter}, heater::Heater, leds, pump, temperature::{self, Temperature}
}, logic::temperature_pid::TemperaturePID};
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(mut spawner: Spawner) -> ! {
    let _p = embassy_stm32::init(Default::default());
    let mut pump = unsafe { pump::Pump::new() };

    let mut flow_meter = unsafe { FlowMeter::new(&mut spawner) };
    flow_meter.enable();

    let mut temperatur = unsafe { Temperature::new(&mut spawner) };

    let mut heater = unsafe { Heater::new(&mut spawner) };

    let mut buttons = unsafe { buttons::Buttons::new(&mut spawner) };
    let mut leds = unsafe { leds::LEDs::new(&mut spawner) };
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

    let mut pid = TemperaturePID::new();
    let mut start_flowed_value = 0;

    pid.set_target_temperature(63);

    loop {
        let event = select(
            buttons.wait_for_button_state_change(),
            Timer::after_millis(50),
        );
        match event.await {
            embassy_futures::select::Either::First(event) => {
                let new_state = event.new_state().state();
                let source = event.new_state().source();
                info!("Button {:?} is now in state {:?}", &source, new_state);

                match source {
                    buttons::ButtonKind::OneCup => {
                        if new_state == ButtonState::Pressed {
                            leds.set_state(leds::LEDKind::OneCup, leds::LEDState::Blinking(2));
                            pump.set_power(pump::PumpPower::Fraction(0.5));
                            start_flowed_value = flow_meter.flowed_mg();
                            pump.enable();
                        }
                    }
                    buttons::ButtonKind::TwoCup => {
                        leds.set_state(leds::LEDKind::TwoCup, leds::LEDState::Blinking(3));
                        if new_state == ButtonState::Pressed {
                            leds.set_state(leds::LEDKind::OneCup, leds::LEDState::Blinking(2));
                            pump.set_power(pump::PumpPower::Fraction(1.0));
                            pump.enable();
                            start_flowed_value = flow_meter.flowed_mg();
                            pump.enable();
                        }
                    }
                    buttons::ButtonKind::Steam => {
                        pid.set_target_temperature(0);
                    }
                    buttons::ButtonKind::HotWater => {
                        pid.set_target_temperature(0);
                    }
                    _ => (),
                }
            }
            embassy_futures::select::Either::Second(_) => {
                let temperature = temperatur.temperature_in_c();
                info!("temperatur={}°C", temperature);
                let next_value = pid.update(temperature);
                info!("pid_next_power_value={}", next_value);
                heater.set_power(next_value);
                if flow_meter.flowed_mg() - start_flowed_value > 100000 {
                    pump.disable();
                }
            }
        }
    }
}
