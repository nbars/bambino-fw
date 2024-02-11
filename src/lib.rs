//!
//! A library used to control the hardware of an Sage/Breville Bambino (BES450) portafilter machine.
//!
#![no_std]
#![warn(
    clippy::cognitive_complexity,
)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::empty_structs_with_brackets,
    arithmetic_overflow,
    clippy::missing_panics_doc,
    clippy::semicolon_if_nothing_returned,
)]

pub mod buttons;
pub mod leds;
pub mod pump;
