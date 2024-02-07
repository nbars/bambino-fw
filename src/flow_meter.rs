//!
//! Module to read the ODE AB32 flow meter that is used to measure the
//! amount of water flowwing through the pump.
//!

use embassy_stm32::{
    gpio::{self, AnyPin},
    rcc::low_level::RccPeripheral,
    timer::low_level::{Basic16bitInstance, GeneralPurpose16bitInstance},
};

pub struct FlowMeter<'a> {
    flow_enable_pin: gpio::Output<'a, AnyPin>,
}

pub struct MilliliterPerSecond(pub u8);

// TODO: Measure flow and clibrate.
// flow measurements
// 171.5g / 353 events

impl<'a> FlowMeter<'a> {
    pub fn new(flow_enable_pin: AnyPin, signal_pin: AnyPin) -> Self {
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
                v.set_ccnp(1, false)
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

        FlowMeter { flow_enable_pin }
    }

    pub fn current_flow(&mut self) -> MilliliterPerSecond {
        todo!();
    }

    pub fn enable(&mut self) {
        embassy_stm32::peripherals::TIM3::regs()
            .cnt()
            .write_value(embassy_stm32::pac::timer::regs::Cnt16(0));
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .cr1()
            .modify(|r| r.set_cen(true));
        self.flow_enable_pin.set_high();
    }

    pub fn disable(&mut self) {
        self.flow_enable_pin.set_low();
        embassy_stm32::peripherals::TIM3::regs_gp16()
            .cr1()
            .modify(|r| r.set_cen(false));
    }
}
