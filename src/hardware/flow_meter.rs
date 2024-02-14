//!
//! Module to read the ODE AB32 flow meter that is used to measure the
//! amount of water flowwing through the pump.
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
use portable_atomic::AtomicU32;

static TOTAL_FLOW_IN_MG_SIGNAL: Signal<ThreadModeRawMutex, u32> = Signal::new();
static TOTAL_FLOW_IN_MG: AtomicU32 = AtomicU32::new(0);

// TODO: Compensate for different pump speeds.
const MG_PER_PULSE: u32 = 416;

/// The flow meter of the machine used to measure the water flow.
pub struct FlowMeter;

impl FlowMeter {
    /// Create a new `FlowMeter` instance.
    ///
    /// # Safety
    /// This is only safe to be called once.
    ///
    /// # Panics
    /// If there is not enough memory to spawn a new task.
    pub unsafe fn new(spawner: &mut Spawner) -> Self {
        spawner.spawn(flowmeter_task()).unwrap();
        FlowMeter {}
    }

    /// The amount of water flowed so far.
    pub fn flowed_mg(&self) -> u32 {
        TOTAL_FLOW_IN_MG.load(portable_atomic::Ordering::SeqCst)
    }

    /// Wait until the flowed milligram value received an update and return the new value.
    pub async fn wait_for_next_flowd_mg(&self) -> u32 {
        TOTAL_FLOW_IN_MG_SIGNAL.wait().await
    }

    /// Wait until the specified amount of water has been poured.
    pub async fn wait_for_amount(&self, amount_in_mg: u32) {
        let start_mg = self.flowed_mg();
        loop {
            let new_value = self.wait_for_next_flowd_mg().await;
            if new_value.wrapping_sub(start_mg) >= amount_in_mg {
                return;
            }
        }
    }
}

struct FlowMeterTask<'a> {
    flow_enable: gpio::Output<'a, AnyPin>,
    signal: ExtiInput<'a, AnyPin>,
}

impl<'a> FlowMeterTask<'a> {
    /// Create a new `FlowMeter`` instance.
    pub fn new() -> Self {
        let p = unsafe { Peripherals::steal() };

        let flow_enable_pin = p.PB11.degrade();
        let signal_input: AnyPin = p.PA7.degrade();

        let flow_enable = gpio::Output::new(flow_enable_pin, gpio::Level::Low, gpio::Speed::Low);

        let signal_input = gpio::Input::new(signal_input.degrade(), gpio::Pull::None);
        let signal = ExtiInput::new(signal_input, p.EXTI4.degrade());

        FlowMeterTask {
            flow_enable,
            signal,
        }
    }

    async fn wait_for_pulse(&mut self) {
        self.signal.wait_for_falling_edge().await;
    }

    fn enable(&mut self) {
        self.flow_enable.set_high();
    }

    fn disable(&mut self) {
        self.flow_enable.set_low();
    }
}

#[embassy_executor::task]
async fn flowmeter_task() -> ! {
    let mut flow_meter = FlowMeterTask::new();
    flow_meter.enable();

    loop {
        flow_meter.wait_for_pulse().await;
        let amount_mg = MG_PER_PULSE;
        let new_amount_mg =
            TOTAL_FLOW_IN_MG.fetch_add(amount_mg, portable_atomic::Ordering::SeqCst);
        TOTAL_FLOW_IN_MG_SIGNAL.signal(new_amount_mg);
    }
}
