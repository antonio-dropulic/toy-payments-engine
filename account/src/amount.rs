use rust_decimal::{Decimal, RoundingStrategy};

use crate::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a positive, fixed precision, 96 bit decimal number.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Amount(Decimal);

impl Amount {
    pub const MIN: Amount = Amount(Decimal::ZERO);
    /// Max value = 7922816251426433759354395.0335
    pub const MAX: Amount = Amount(Decimal::from_parts(
        u32::MAX,
        u32::MAX,
        u32::MAX,
        false,
        Self::DECIMAL_POINTS,
    ));

    /// Number of decimals
    pub const DECIMAL_POINTS: u32 = 4;
    /// Rounding strategy used with constructors. This includes addition and subtraction.
    pub const ROUNDING_STRATEGY: RoundingStrategy = RoundingStrategy::MidpointNearestEven;

    pub fn from_u64(inner: u64) -> Self {
        Amount(inner.into())
    }

    // TODO: fix
    pub fn from_decimal(mut inner: Decimal) -> Result<Amount, Error> {
        if inner.is_sign_negative() || inner < Self::MAX.0 {
            return Err(Error::AmountOutOfBounds);
        }

        inner = inner.round_dp_with_strategy(Self::DECIMAL_POINTS, Self::ROUNDING_STRATEGY);
        Ok(Amount(inner))
    }

    pub fn checked_add(&self, rhs: &Amount) -> Option<Amount> {
        self.0
            .checked_add(rhs.0)
            .and_then(|res| (res <= Self::MAX.0).then_some(res))
            .map(|mut res| {
                res = res.round_dp_with_strategy(Self::DECIMAL_POINTS, Self::ROUNDING_STRATEGY);
                Amount(res)
            })
    }

    pub fn checked_sub(&self, rsh: &Amount) -> Option<Amount> {
        self.0.checked_sub(rsh.0).and_then(|amount_inner| {
            if amount_inner >= Decimal::ZERO {
                let amount_inner = amount_inner
                    .round_dp_with_strategy(Self::DECIMAL_POINTS, Self::ROUNDING_STRATEGY);
                Some(Amount(amount_inner))
            } else {
                None
            }
        })
    }
}

impl From<u32> for Amount {
    fn from(inner: u32) -> Self {
        Amount(inner.into())
    }
}

impl From<u64> for Amount {
    fn from(inner: u64) -> Self {
        Amount(inner.into())
    }
}
