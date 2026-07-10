use serde::Serialize;

use crate::{NexusError, NexusResult};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct Amount(u128);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct Bps(u16);

impl Amount {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub fn new(units: u128) -> NexusResult<Self> {
        Ok(Self(units))
    }

    pub const fn units(self) -> u128 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, rhs: Self) -> NexusResult<Self> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(NexusError::AmountOverflow)
    }

    pub fn checked_sub(self, rhs: Self) -> NexusResult<Self> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(NexusError::AmountUnderflow)
    }

    pub fn checked_mul(self, rhs: u128) -> NexusResult<Self> {
        self.0
            .checked_mul(rhs)
            .map(Self)
            .ok_or(NexusError::AmountOverflow)
    }

    pub fn checked_div(self, rhs: u128) -> NexusResult<Self> {
        if rhs == 0 {
            return Err(NexusError::DivisionByZero);
        }
        Ok(Self(self.0 / rhs))
    }

    pub fn checked_mul_bps(self, bps: Bps) -> NexusResult<Self> {
        self.0
            .checked_mul(u128::from(bps.units()))
            .and_then(|value| value.checked_div(10_000))
            .map(Self)
            .ok_or(NexusError::AmountOverflow)
    }

    pub fn checked_mul_ratio(self, numerator: u128, denominator: u128) -> NexusResult<Self> {
        if denominator == 0 {
            return Err(NexusError::DivisionByZero);
        }
        self.0
            .checked_mul(numerator)
            .and_then(|value| value.checked_div(denominator))
            .map(Self)
            .ok_or(NexusError::AmountOverflow)
    }

    pub fn min(self, rhs: Self) -> Self {
        if self <= rhs { self } else { rhs }
    }
}

impl Bps {
    pub fn new(units: u16) -> NexusResult<Self> {
        if units > 10_000 {
            return Err(NexusError::BpsOutOfRange(units));
        }
        Ok(Self(units))
    }

    pub const fn units(self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::fmt::Display for Bps {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
