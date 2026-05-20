use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

macro_rules! define_pool_type {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default,
        )]
        #[serde(transparent)]
        pub struct $name(i16);

        impl $name {
            pub const ZERO: Self = Self(0);

            #[inline]
            pub const fn new(value: i16) -> Self {
                Self(value)
            }

            #[inline]
            pub const fn get(self) -> i16 {
                self.0
            }

            #[inline]
            pub const fn is_positive(self) -> bool {
                self.0 > 0
            }
        }

        impl Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0.saturating_add(rhs.0))
            }
        }

        impl Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0.saturating_sub(rhs.0))
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self.0.saturating_add(rhs.0);
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 = self.0.saturating_sub(rhs.0);
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_pool_type!(HitPoints, "Очки здоровья (HP). Могут быть < 0.");
define_pool_type!(PowerPoints, "Очки магии/энергии (MP).");
define_pool_type!(FatiguePoints, "Очки усталости (FP).");
define_pool_type!(SanityPoints, "Очки рассудка (SAN).");
