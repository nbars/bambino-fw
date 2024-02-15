//!
//! Module to read the ODE AB32 flow meter that is used to measure the
//! amount of water flowing through the pump.
//!
#![allow(clippy::new_without_default)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{Channel as _, ExtiInput};
use embassy_stm32::{
    gpio::{self, AnyPin, Pin},
    Peripherals,
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant};
use portable_atomic::AtomicU32;

static TOTAL_FLOW_IN_MG_SIGNAL: Signal<ThreadModeRawMutex, u32> = Signal::new();
static TOTAL_FLOW_IN_MG: AtomicU32 = AtomicU32::new(0);
static PULSE_CTR: AtomicU32 = AtomicU32::new(0);

// Value optimized for 9 pulses per second (pump at ~50% of it's power).
const MG_PER_PULSE: u32 = 440;

/// The flow meter of the machine used to measure the water flow.
pub struct FlowMeter<'a> {
    flow_enable: gpio::Output<'a, AnyPin>,
}

impl<'a> FlowMeter<'a> {
    /// Create a new `FlowMeter` instance.
    ///
    /// # Safety
    /// This is only safe to be called once.
    ///
    /// # Panics
    /// If there is not enough memory to spawn a new task.
    pub unsafe fn new(spawner: &mut Spawner) -> Self {
        let p = unsafe { Peripherals::steal() };
        let flow_enable_pin = p.PB11.degrade();
        let flow_enable = gpio::Output::new(flow_enable_pin, gpio::Level::Low, gpio::Speed::Low);

        spawner.spawn(flowmeter_task()).unwrap();


        FlowMeter {flow_enable}
    }

    /// The amount of water flowed so far.
    pub fn flowed_mg(&self) -> u32 {
        TOTAL_FLOW_IN_MG.load(portable_atomic::Ordering::SeqCst)
    }

    /// Wait until the flowed milligram value received an update and return the new value.
    pub async fn wait_for_next_update(&self) -> u32 {
        defmt::debug_assert!(self.is_enabled());
        TOTAL_FLOW_IN_MG_SIGNAL.wait().await
    }

    /// Wait until the specified amount of water has been poured.
    pub async fn wait_for_amount(&self, amount_in_mg: u32) {
        defmt::debug_assert!(self.is_enabled());
        let start_mg = self.flowed_mg();
        loop {
            let new_value = self.wait_for_next_update().await;
            if new_value.wrapping_sub(start_mg) >= amount_in_mg {
                return;
            }
        }
    }

    /// Number of pulses counted so far.
    pub fn pulse_ctr(&self) -> u32 {
        PULSE_CTR.load(portable_atomic::Ordering::SeqCst)
    }

    /// Check whether the flow meter is powered.
    pub fn is_enabled(&self) -> bool {
        self.flow_enable.is_set_high()
    }

    /// Enable the power supply for the flow meter.
    pub fn enable(&mut self) {
        self.flow_enable.set_high();
    }

    /// Disable the power supply for the flow meter.
    pub fn disable(&mut self) {
        self.flow_enable.set_low();
    }
}

struct FlowMeterTask<'a> {
    signal: ExtiInput<'a, AnyPin>,
}

impl<'a> FlowMeterTask<'a> {
    /// Create a new `FlowMeter`` instance.
    pub fn new() -> Self {
        let p = unsafe { Peripherals::steal() };

        let signal_input: AnyPin = p.PA7.degrade();

        let signal_input = gpio::Input::new(signal_input.degrade(), gpio::Pull::None);
        let signal = ExtiInput::new(signal_input, p.EXTI4.degrade());

        FlowMeterTask {
            signal,
        }
    }

    async fn wait_for_pulse(&mut self) {
        self.signal.wait_for_falling_edge().await;
    }

}

#[embassy_executor::task]
async fn flowmeter_task() -> ! {
    let mut flow_meter = FlowMeterTask::new();
    let mut pulses_per_second: u32 = 0;

    loop {
        let before_pulse = Instant::now();
        flow_meter.wait_for_pulse().await;
        let pulse_duration = before_pulse.elapsed();
        if pulse_duration < Duration::from_secs(1) {
            pulses_per_second = (4 * pulses_per_second + (1000 / pulse_duration.as_millis() as u32)) / 5;
        } else {
            pulses_per_second = 0;
        }

        /*
        Goal was to pour 50 g (theoretically 115 pulses with MG_PER_PULSE being 440) of water.
        These are the measurements for different pump speeds.

        Pulses/s | out      | Difference per pulse
        5        | 40.3 g  | (50-40.3) / 115 = +0.0843g   = +84.3 mg
        9        | 49.8 g    | (50-49.8) / 115 = +0.0017g =  +1.7 mg
        13       | 57.4 g  | (50-57.4) / 115 = -0.0643g   = -64.3 mg
        https://www.wolframalpha.com/input?i=linear+fit+calculator&assumption=%7B%22F%22%2C+%22LinearFitCalculator%22%2C+%22data2%22%7D+-%3E%22%7B%285%2C+84.3%29%2C+%289%2C+1.7%29%2C+%2813%2C-64.3%29%7D%22
        */

        // The amount of water that is purred more/less compared to the amount defined in MG_PER_PULSE.
        let correction_amount_mg = (174.408 - 18.575 * pulses_per_second as f32) as i32;

        // compensate
        let amount_mg = MG_PER_PULSE as i32 - correction_amount_mg;
        let new_amount_mg =
            TOTAL_FLOW_IN_MG.fetch_add(amount_mg.try_into().unwrap(), portable_atomic::Ordering::SeqCst);
        PULSE_CTR.add(1, portable_atomic::Ordering::SeqCst);
        TOTAL_FLOW_IN_MG_SIGNAL.signal(new_amount_mg);
    }
}
