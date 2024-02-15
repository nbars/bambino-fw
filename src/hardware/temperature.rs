//!
//! This module contains the logic related to the temperature sensor that allows
//! to measure the water temperature just before it is exiting the heater.
//!

use core::num::NonZeroU16;

use embassy_executor::{Executor, Spawner};
use embassy_stm32::{
    adc::{self, Adc},
    bind_interrupts,
    peripherals::{ADC, PB1},
    Peripherals,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Delay, Timer};
use portable_atomic::AtomicU32;

static RAW_TEMPERATURE_SIGNAL: Signal<ThreadModeRawMutex, u32> = Signal::new();
static RAW_TEMPERATURE_C: AtomicU32 = AtomicU32::new(0);

pub struct Temperature;

impl Temperature {

    /// Create a new `Temperature` instance.
    ///
    /// # Safety
    /// This is only safe to be called once.
    ///
    /// # Panics
    /// If there is not enough memory to spawn a new task.
    pub unsafe fn new(spawner: &mut Spawner) -> Self {
        spawner.spawn(temperature_task()).unwrap();

        Temperature {

        }
    }

    pub fn temperature_in_c(&self) -> u32 {
        let raw_value = RAW_TEMPERATURE_C.load(portable_atomic::Ordering::Relaxed);
        Temperature::raw_into_celsius(raw_value)
    }

    fn raw_into_celsius(raw_value: u32) -> u32 {
        /*
        ADC Value -> Temperature
        1000.0 -> 17
        1339 -> 25
        2064 -> 44
        2997 -> 71
        3341 -> 81

        https://www.wolframalpha.com/input?i=quadratic+fit+calculator&assumption=%7B%22F%22%2C+%22QuadraticFitCalculator%22%2C+%22data2%22%7D+-%3E%22%7B%281000%2C+17%29%2C+%281339%2C25%29%2C+%282064%2C44%29%2C+%282997%2C71%29%2C+%283341%2C+81%29%7D%22
        1.50104×10^-6 x^2 + 0.0209623 x - 5.59606
        */
        let raw_value = raw_value as f32;
        let f1 = 1.50104f32 * (1f32 / (10u64.pow(6) as f32));
        let result = f1 * (raw_value * raw_value) + 0.0209623f32 * raw_value - 5.59606f32;
        result as u32
    }

}



struct TemperatureTask<'a> {
    adc: Adc<'a, ADC>,
    ntc_pin: PB1,
}

impl<'a> TemperatureTask<'a> {
    /// Create a new `Temperature` instance in order to measure the water temperature.
    /// # Safety
    /// This is only safe when called once and without concurrently calling any of the `new()``
    /// methods of the other hardware components.
    unsafe fn new() -> Self {
        let p = unsafe { Peripherals::steal() };

        bind_interrupts!(struct Irqs {
            ADC1 => adc::InterruptHandler<ADC>;
        });
        let mut adc = Adc::new(p.ADC, Irqs, &mut Delay);
        // TODO: Check sample time.
        adc.set_sample_time(adc::SampleTime::Cycles55_5);
        let _vref = adc.enable_vref(&mut Delay);

        TemperatureTask {
            adc,
            ntc_pin: p.PB1,
        }
    }

    async fn read_raw_averaged(&mut self, iterations: NonZeroU16) -> u32 {
        let mut mean: u32 = 0;
        for _ in 0..iterations.get() {
            let reading = self.adc.read(&mut self.ntc_pin).await;
            mean += reading as u32;
        }

        mean / iterations.get() as u32
    }
}

#[embassy_executor::task]
async fn temperature_task() -> ! {
    let mut task = unsafe { TemperatureTask::new() };

    loop {
        let raw_temperature = task.read_raw_averaged(NonZeroU16::new(10).unwrap()).await;
        RAW_TEMPERATURE_C.store(raw_temperature, portable_atomic::Ordering::Relaxed);
        RAW_TEMPERATURE_SIGNAL.signal(raw_temperature);
        Timer::after_millis(10).await;
    }
}