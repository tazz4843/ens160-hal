// #![allow(incomplete_features)]
// #![feature(generic_const_exprs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod ens160_impl;
pub mod error;
mod registers;

use core::{
    convert::TryFrom,
    ops::{Deref, DerefMut},
};

use bitfield::bitfield;
pub use ens160_impl::Ens160;
use error::AirqualityConvError;

/// Commands for ENS160_COMMAND_REG.
#[repr(u8)]
enum Command {
    /// No operation
    Nop = 0x00,
    /// Get FW version
    GetAppVersion = 0x0E,
    /// Clears GPR Read Registers
    Clear = 0xCC,
}

/// Operation Mode of the sensor.
#[repr(u8)]
enum OperationMode {
    /// DEEP SLEEP mode (low power standby).
    Sleep = 0x00,
    /// IDLE mode (low-power).
    Idle = 0x01,
    /// STANDARD Gas Sensing Modes.
    Standard = 0x02,
    /// Reset device.
    Reset = 0xF0,
}

bitfield! {
    /// Status of the sensor.
    pub struct Status(u8);
    impl Debug;
    pub bool, running_normally, _: 7;
    pub bool, error, _: 6;
    pub into Validity, validity_flag, _: 3,2;
    pub bool, data_is_ready, _: 1;
    pub bool, new_data_in_gpr, _: 0;
}

// #[derive(BitfieldSpecifier)]
#[derive(Debug, Clone, Copy)]
pub enum Validity {
    NormalOperation,
    WarmupPhase,
    InitStartupPhase,
    InvalidOutput,
}

impl From<u8> for Validity {
    fn from(v: u8) -> Self {
        match v {
            0b00 => Self::NormalOperation,
            0b01 => Self::WarmupPhase,
            0b10 => Self::InitStartupPhase,
            0b11 => Self::InvalidOutput,
            _ => unreachable!(),
        }
    }
}

bitfield! {
    #[derive(Default)]
    struct InterruptRegister(u8);
    impl Debug;
    from into InterruptState, _, set_interrupt_state: 6, 6;
    from into PinMode,  ddfsdf, set_pin_mode: 5, 5;
    bool,  _, set_on_data_in_gpr_register: 3;
    bool,  _, set_on_data_in_data_register: 1;
    bool,  _, set_enabled: 0;
}

// #[derive(BitfieldSpecifier)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    OpenDrain,
    PushPull,
}

impl From<PinMode> for u8 {
    fn from(m: PinMode) -> u8 {
        match m {
            PinMode::OpenDrain => 0x0,
            PinMode::PushPull => 0x1,
        }
    }
}

impl From<u8> for PinMode {
    fn from(b: u8) -> Self {
        match b {
            0x1 => Self::PushPull,
            _ => Self::OpenDrain,
        }
    }
}

// #[derive(BitfieldSpecifier)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptState {
    ActiveLow,
    ActiveHigh,
}

impl From<InterruptState> for u8 {
    fn from(i: InterruptState) -> u8 {
        match i {
            InterruptState::ActiveHigh => 0x1,
            InterruptState::ActiveLow => 0x0,
        }
    }
}

impl From<u8> for InterruptState {
    fn from(b: u8) -> Self {
        match b {
            0x1 => Self::ActiveHigh,
            _ => Self::ActiveLow,
        }
    }
}

#[derive(Debug, Default)]
pub struct InterruptConfig(InterruptRegister);

impl InterruptConfig {
    pub fn set_pin_interrupt_state(mut self, state: InterruptState) -> Self {
        self.0.set_interrupt_state(state);
        self
    }

    pub fn enable_for_measure_data_is_ready(mut self) -> Self {
        self.0.set_enabled(true);
        self.0.set_on_data_in_data_register(true);
        self
    }

    pub fn enable_for_data_in_read_register(mut self) -> Self {
        self.0.set_enabled(true);
        self.0.set_on_data_in_gpr_register(true);
        self
    }

    pub fn set_pin_mode(mut self, mode: PinMode) -> Self {
        self.0.set_pin_mode(mode);
        self
    }

    fn finish(self) -> InterruptRegister {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum AirQualityIndex {
    Excellent = 1,
    Good = 2,
    Moderate = 3,
    Poor = 4,
    Unhealthy = 5,
}

impl From<u8> for AirQualityIndex {
    fn from(i: u8) -> Self {
        match i {
            1 => Self::Excellent,
            2 => Self::Good,
            3 => Self::Moderate,
            4 => Self::Poor,
            5 => Self::Unhealthy,
            _ => Self::Unhealthy,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ECo2(u16);

impl From<u16> for ECo2 {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl TryFrom<ECo2> for AirQualityIndex {
    type Error = AirqualityConvError;

    fn try_from(e: ECo2) -> Result<Self, Self::Error> {
        let value = e.0;
        match value {
            400..=599 => Ok(Self::Excellent),
            600..=799 => Ok(Self::Good),
            800..=999 => Ok(Self::Moderate),
            1000..=1499 => Ok(Self::Poor),
            1500..=u16::MAX => Ok(Self::Unhealthy),
            _ => Err(AirqualityConvError(value)),
        }
    }
}

impl Deref for ECo2 {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ECo2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {

    use crate::{InterruptConfig, PinMode, Status, Validity};

    #[test]
    fn test_status_register_layout() {
        let status = Status(0b00000001);
        assert!(status.new_data_in_gpr());

        let status = Status(0b10000100);
        assert!(status.running_normally());
        assert!(matches!(status.validity_flag(), Validity::WarmupPhase));

        let status = Status(0b00001110);
        assert!(status.data_is_ready());
        assert!(matches!(status.validity_flag(), Validity::InvalidOutput))
    }

    #[test]
    fn test_interrupt_config() {
        let config = InterruptConfig::default()
            .enable_for_measure_data_is_ready()
            .set_pin_mode(PinMode::PushPull)
            .finish();
        assert_eq!(config.0, 0b00100011)
    }

    #[test]
    fn test_byte_order() {
        let b: u16 = 0x10;
        assert_eq!(b.to_be_bytes(), [0x0, 0x10])
    }
}

#[cfg(all(feature = "blocking", feature = "async"))]
compile_error!("Cannot enable both `blocking` and `async` features");
