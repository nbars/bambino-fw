//!
//! A library used to control the hardware of an Sage/Breville Bambino (BES450) portafilter machine.
//!
#![no_std]
#![no_main]
#![allow(clippy::new_without_default)]
#![warn(clippy::cognitive_complexity, missing_docs)]
#![deny(
    clippy::missing_safety_doc,
    clippy::empty_structs_with_brackets,
    arithmetic_overflow,
    clippy::missing_panics_doc,
    clippy::semicolon_if_nothing_returned
)]

pub mod hardware;
pub mod logic;
