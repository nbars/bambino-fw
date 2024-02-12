//!
//! Module to read the ODE AB32 flow meter that is used to measure the
//! amount of water flowwing through the pump.
//!
#![allow(clippy::new_without_default)]

use cortex_m::interrupt;
use embassy_executor::Spawner;
use embassy_stm32::interrupt::Interrupt;
use embassy_stm32::pac::timer::regs::Cnt16;
use embassy_stm32::{
    gpio::{self, AnyPin, Pin},
    peripherals,
    rcc::low_level::RccPeripheral,
    timer::low_level::{
        Basic16bitInstance, CaptureCompare16bitInstance, GeneralPurpose16bitInstance,
    },
    Peripheral, Peripherals,
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Ticker};

// #[interrupt]
// unsafe fn TIM3() {
//     EXECUTOR_MED.on_interrupt()
// }

/// The SI unit milliliter per seconds.
pub struct MilliliterPerSecond(pub u32);

static CURRENT_FLOW: Signal<ThreadModeRawMutex, MilliliterPerSecond> = Signal::new();

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
}


struct FlowMeterTask<'a> {
    flow_enable_pin: gpio::Output<'a, AnyPin>,
}

impl<'a> FlowMeterTask<'a> {
    /// Create a new `FlowMeter`` instance.
    pub fn new() -> Self {
        let p = unsafe { Peripherals::steal() };

        let flow_enable_pin = p.PB11.degrade();
        let signal_pin: AnyPin = p.PA7.degrade();

        let flow_enable_pin =
            gpio::Output::new(flow_enable_pin, gpio::Level::Low, gpio::Speed::Low);

        let block = signal_pin.block();
        block
            .moder()
            .modify(|r| r.set_moder(7, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE));
        block.afr(0).modify(|r| r.set_afr(7, 1));

        embassy_stm32::peripherals::TIM3::enable_and_reset();
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .ccmr_input(0)
            .modify(|r| r.set_ccs(1, embassy_stm32::pac::timer::vals::CcmrInputCcs::TI4));
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .ccer()
            .modify(|v| {
                v.set_ccp(1, false);
                v.set_ccnp(1, false);
            });
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .smcr()
            .modify(|v| v.set_sms(embassy_stm32::pac::timer::vals::Sms::EXT_CLOCK_MODE));
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .smcr()
            .modify(|v| v.set_ts(embassy_stm32::pac::timer::vals::Ts::TI2FP2));
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .arr()
            .modify(|w| w.set_arr(u16::MAX));
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .cr1()
            .modify(|r| r.set_cen(true));

        // todo: overflow interrupt

        FlowMeterTask { flow_enable_pin }
    }

    fn read_counter_and_reset(&mut self) -> u16 {
        embassy_stm32::peripherals::TIM3::regs()
            // FIXME: This is likely not atomic :/
            .cnt()
            .modify(|f| {
                let old = f.0;
                *f = Cnt16(0);
                old
            })
            .try_into()
            .unwrap()
    }

    fn counter_to_mg(counter: u32) -> f32 {
        counter as f32 * 0.4858
    }

    fn enable(&mut self) {
        embassy_stm32::peripherals::TIM3::regs()
            .cnt()
            .write_value(embassy_stm32::pac::timer::regs::Cnt16(0));
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .cr1()
            .modify(|r| r.set_cen(true));
        self.flow_enable_pin.set_high();
    }

    fn disable(&mut self) {
        self.flow_enable_pin.set_low();
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .cr1()
            .modify(|r| r.set_cen(false));
    }

    fn update_flow(&mut self, elapsed_time: Duration) {
        todo!();
        let ctr = self.read_counter_and_reset();
        //let flow = FlowMeterTask::counter_to_mg(ctr);
    }
}

#[embassy_executor::task]
async fn flowmeter_task() -> ! {
    let mut flow_meter = FlowMeterTask::new();
    let mut ticker = Ticker::every(Duration::from_millis(100));

    CURRENT_FLOW.signal(MilliliterPerSecond(0));

    loop {
        let last_update = Instant::now();
        ticker.next();

        let elapsed = last_update.elapsed();
        flow_meter.update_flow(elapsed);
    }
}
