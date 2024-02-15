//!
//! Module to control the heater.
//!

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{self, select};
use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    Peripherals,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker, Timer};

static DUTY_CYCLE: Signal<ThreadModeRawMutex, u32> = Signal::new();


pub struct Heater;

impl Heater {

    /// Create a new `Heater` instance.
    ///
    /// # Safety
    /// This is only safe to be called once.
    ///
    /// # Panics
    /// If there is not enough memory to spawn a new task.
    pub unsafe fn new(spawner: &mut Spawner) -> Self {
        spawner.spawn(heater_task()).unwrap();

        Heater {

        }
    }

    pub fn set_power(&mut self, power_in_percent: u32) {
        assert!(power_in_percent <= 100);
        DUTY_CYCLE.signal(power_in_percent);
    }
}

impl Drop for Heater {
    fn drop(&mut self) {
        DUTY_CYCLE.signal(0);
    }
}


/// The heater used to boil the water.
struct HeaterTask<'a> {
    pin: Output<'a, AnyPin>,
}

impl<'a> HeaterTask<'a> {
    /// Create a new heater instance.
    /// # Safety
    /// This is only safe when called once and without concurrently calling any of the `new()``
    /// methods of the other hardware components.
    unsafe fn new() -> Self {
        let p = Peripherals::steal();
        let pin = Output::new(p.PB6.degrade(), Level::Low, Speed::Low);
        HeaterTask { pin }
    }

    /// Turn the heater on.
    fn on(&mut self) {
        self.pin.set_high();
    }

    /// Turn the heater off.
    fn off(&mut self) {
        self.pin.set_low();
    }
}

impl<'a> Drop for HeaterTask<'a> {
    fn drop(&mut self) {
        self.off();
    }
}

#[embassy_executor::task]
async fn heater_task() -> ! {
    let mut heater = unsafe { HeaterTask::new() };
    heater.off();

    let frequency = Duration::from_hz(10);
    let mut ticker = Ticker::every(frequency);
    let mut current_duty_cycle = None;

    loop {
        let new_duty_cycle = DUTY_CYCLE.wait();

        match select::select(new_duty_cycle, ticker.next()).await {
            select::Either::First(new_duty_cycle) => {
                if new_duty_cycle > 0 {
                    let duty_cycle_ms = frequency.as_millis() as f32 * (new_duty_cycle as f32 / 100f32);
                    current_duty_cycle =  Some(duty_cycle_ms as u32);
                } else {
                    current_duty_cycle = None;
                }
            },
            select::Either::Second(_) => {
                if let Some(current_duty_cycle) = current_duty_cycle {
                    heater.on();
                    info!("current_duty_cycle={}", current_duty_cycle);
                    Timer::after_millis(current_duty_cycle as u64).await;
                    heater.off();
                }
            },
        }

    }
}