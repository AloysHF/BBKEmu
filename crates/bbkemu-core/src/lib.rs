//! BBKEmu Core - Platform-independent BBK game emulator engine
//!
//! This crate implements a BBK A-series electronic dictionary game emulator.

pub mod audio;
pub mod cheat;
pub mod cpu;
pub mod debug;
pub mod emulator;
pub mod firmware;
pub mod font_data;
pub mod gam;
pub mod input;
pub mod lcd;
pub mod memory;
pub mod model;
pub mod save;

pub use emulator::Emulator;
pub use model::BbkModel;
