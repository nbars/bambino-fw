//!
//! This module contains the logic related to the temperature sensor that allows
//! to measure the water temperature just before it is exiting the heater.
//!

use core::num::NonZeroU16;

use embassy_stm32::{
    adc::{self, Adc},
    bind_interrupts,
    peripherals::{ADC, PB1},
    Peripherals,
};
use embassy_time::Delay;

pub struct Temperature<'a> {
    adc: Adc<'a, ADC>,
    ntc_pin: PB1,
}

impl<'a> Temperature<'a> {
    /// Create a new `Temperature` instrance in order to measure the water temperature.
    /// # Safety
    /// This is only safe when called once and without concurrently calling any of the `new()``
    /// methods of the other hardware components.
    pub unsafe fn new() -> Self {
        let p = unsafe { Peripherals::steal() };

        bind_interrupts!(struct Irqs {
            ADC1 => adc::InterruptHandler<ADC>;
        });
        let mut adc = Adc::new(p.ADC, Irqs, &mut Delay);
        adc.set_sample_time(adc::SampleTime::Cycles239_5);
        let _vref = adc.enable_vref(&mut Delay);

        Temperature {
            adc,
            ntc_pin: p.PB1,
        }
    }

    pub async fn read_raw_averaged(&mut self, iterations: NonZeroU16) -> f32 {
        let mut mean: u64 = 0;
        for _ in 0..iterations.get() {
            let reading = self.adc.read(&mut self.ntc_pin).await;
            mean += reading as u64;
        }

        mean as f32 / iterations.get() as f32
    }

    pub async fn read_averaged_celcius(&mut self, iterations: NonZeroU16) -> f64 {
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
        let raw_mean = self.read_raw_averaged(iterations).await as f64;
        let f1 = 1.50104f64 * (1f64 / (10u64.pow(6) as f64));
        f1 * (raw_mean * raw_mean) + 0.0209623f64 * raw_mean - 5.59606f64
    }
}
