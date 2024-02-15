use core::time::Duration;

use embassy_time::Instant;

use crate::hardware::temperature;


#[allow(non_snake_case)]
struct PidParameters {
    P: f32,
    I: f32,
    D: f32,
}

const PARAMETERS: PidParameters = PidParameters {
    P: 4.4,
    I: 0.0,
    D: 0.0,
};

const MAX_OUTPUT_VALUE: u32 = 100;


pub struct TemperaturePID {
    last_update: Option<Instant>,
    last_temperature: Option<u32>,
    target_temperatur: u32,
    error: i32,
}

impl TemperaturePID {

    pub fn new() -> Self {
        TemperaturePID {
            last_update: None,
            last_temperature: None,
            target_temperatur: 0,
            error: 0,
        }
    }

    pub fn set_target_temperature(&mut self, target_temperature: u32) {
        self.target_temperatur = target_temperature;
        self.error = 0;
        self.last_update = None;
        self.last_temperature = None;
    }

    pub fn update(&mut self, current_temperature: u32) -> u32 {
        let since_last_update = self.last_update.map(|e| e.elapsed());
        let difference = self.target_temperatur as i32 - current_temperature as i32;

        if let Some(since_last_update) = since_last_update {
            let error = difference * since_last_update.as_millis() as i32;
            if (self.error + error).abs() < 100 {
                self.error += error;
            }
        }

        self.last_temperature = Some(current_temperature);
        self.last_update = Some(Instant::now());

        let output = difference as f32 * PARAMETERS.P + self.error as f32 * PARAMETERS.I;
        let output = 20 + output as i32;
        if output < 0 {
            0
        } else if output > MAX_OUTPUT_VALUE.try_into().unwrap() {
            MAX_OUTPUT_VALUE
        } else  {
            output as u32
        }
    }





}