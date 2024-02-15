#![no_std]
#![no_main]

use bambino_fw::hardware::{
    buttons::{self, ButtonState},
    flow_meter::FlowMeter,
    leds, pump,
};
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

    loop {
        let event = select(
            buttons.wait_for_button_state_change(),
            Timer::after_millis(100),
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
                            let pulses_before = flow_meter.pulse_ctr();
                            pump.enable();
                            flow_meter.wait_for_amount(50000).await;
                            pump.disable();
                            info!("pulses={}", flow_meter.pulse_ctr() - pulses_before);
                        }
                    }
                    buttons::ButtonKind::TwoCup => {
                        leds.set_state(leds::LEDKind::TwoCup, leds::LEDState::Blinking(3));
                        if new_state == ButtonState::Pressed {
                            leds.set_state(leds::LEDKind::OneCup, leds::LEDState::Blinking(2));
                            pump.set_power(pump::PumpPower::Fraction(1.0));
                            let pulses_before = flow_meter.pulse_ctr();
                            pump.enable();
                            flow_meter.wait_for_amount(50000).await;
                            pump.disable();
                            info!("pulses={}", flow_meter.pulse_ctr() - pulses_before);
                        }
                    }
                    _ => (),
                }
            }
            embassy_futures::select::Either::Second(_) => {}
        }
    }
}
