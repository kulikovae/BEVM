// Copyright 2018-2019 Chainpool.

use parity_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

// Substrate
use rstd::{prelude::*, result, slice::Iter};
use support::dispatch::Result;
use support::traits::{Imbalance, SignedImbalance};
use support::StorageMap;

use primitives::traits::{Saturating, Zero};
// ChainX
pub use xr_primitives::{Desc, Memo, Token};

use super::traits::ChainT;
use super::{Module, Trait};

pub use self::imbalances::{NegativeImbalance, PositiveImbalance};

const MAX_TOKEN_LEN: usize = 32;
const MAX_DESC_LEN: usize = 128;

pub type TokenString = &'static [u8];
pub type DescString = TokenString;
pub type Precision = u16;

pub type SignedImbalanceT<T> = SignedImbalance<<T as Trait>::Balance, PositiveImbalance<T>>;

#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub enum Chain {
    ChainX,
    Bitcoin,
    Ethereum,
}

impl Default for Chain {
    fn default() -> Self {
        Chain::ChainX
    }
}

impl Chain {
    pub fn iterator() -> Iter<'static, Chain> {
        static CHAINS: [Chain; 3] = [Chain::ChainX, Chain::Bitcoin, Chain::Ethereum];
        CHAINS.iter()
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Asset {
    token: Token,
    token_name: Token,
    chain: Chain,
    precision: Precision,
    desc: Desc,
}

impl Asset {
    pub fn new(
        token: Token,
        token_name: Token,
        chain: Chain,
        precision: Precision,
        desc: Desc,
    ) -> result::Result<Self, &'static str> {
        let a = Asset {
            token,
            token_name,
            chain,
            precision,
            desc,
        };
        a.is_valid()?;
        Ok(a)
    }
    pub fn is_valid(&self) -> Result {
        is_valid_token(&self.token)?;
        is_valid_token_name(&self.token_name)?;
        is_valid_desc(&self.desc)
    }

    pub fn token(&self) -> Token {
        self.token.clone()
    }
    pub fn token_name(&self) -> Token {
        self.token_name.clone()
    }
    pub fn chain(&self) -> Chain {
        self.chain
    }
    pub fn desc(&self) -> Desc {
        self.desc.clone()
    }
    pub fn set_desc(&mut self, desc: Desc) {
        self.desc = desc
    }
    pub fn precision(&self) -> Precision {
        self.precision
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub enum AssetType {
    Free,
    ReservedStaking,
    ReservedStakingRevocation,
    ReservedWithdrawal,
    ReservedDexSpot,
    ReservedDexFuture,
    ReservedCurrency,
}

// TODO use marco to improve it
impl AssetType {
    pub fn iterator() -> Iter<'static, AssetType> {
        static TYPES: [AssetType; 7] = [
            AssetType::Free,
            AssetType::ReservedStaking,
            AssetType::ReservedStakingRevocation,
            AssetType::ReservedWithdrawal,
            AssetType::ReservedDexSpot,
            AssetType::ReservedDexFuture,
            AssetType::ReservedCurrency,
        ];
        TYPES.iter()
    }
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Free
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub enum AssetErr {
    NotEnough,
    OverFlow,
    TotalAssetNotEnough,
    TotalAssetOverFlow,
    InvalidToken,
    InvalidAccount,
}

impl AssetErr {
    pub fn info(self) -> &'static str {
        match self {
            AssetErr::NotEnough => "balance too low for this account",
            AssetErr::OverFlow => "balance too high for this account",
            AssetErr::TotalAssetNotEnough => "total balance too low for this asset",
            AssetErr::TotalAssetOverFlow => "total balance too high for this asset",
            AssetErr::InvalidToken => "not a valid token for this account",
            AssetErr::InvalidAccount => "account Locked",
        }
    }
}

/// Token can only use numbers (0x30~0x39), capital letters (0x41~0x5A), lowercase letters (0x61~0x7A), -(0x2D), .(0x2E), |(0x7C),  ~(0x7E).
pub fn is_valid_token(v: &[u8]) -> Result {
    if v.len() > MAX_TOKEN_LEN || v.is_empty() {
        return Err("Token length is zero or too long.");
    }
    let is_valid = |c: &u8| -> bool {
        (*c >= 0x30 && *c <= 0x39) // number
            || (*c >= 0x41 && *c <= 0x5A) // capital
            || (*c >= 0x61 && *c <= 0x7A) // small
            || (*c == 0x2D) // -
            || (*c == 0x2E) // .
            || (*c == 0x7C) // |
            || (*c == 0x7E) // ~
    };
    for c in v.iter() {
        if !is_valid(c) {
            return Err(
                "Token can only use numbers, capital/lowercase letters or '-', '.', '|', '~'.",
            );
        }
    }
    Ok(())
}

pub fn is_valid_token_name(v: &[u8]) -> Result {
    if v.len() > MAX_TOKEN_LEN || v.is_empty() {
        return Err("Token name is zero or too long.");
    }
    for c in v.iter() {
        // Visible ASCII char [0x20, 0x7E]
        if *c < 0x20 || *c > 0x7E {
            return Err("Token name can not use an invisiable ASCII char.");
        }
    }
    Ok(())
}

/// Desc can only be Visible ASCII chars.
pub fn is_valid_desc(v: &[u8]) -> Result {
    if v.len() > MAX_DESC_LEN {
        return Err("Token desc too long");
    }
    for c in v.iter() {
        // Visible ASCII char [0x20, 0x7E]
        if *c < 0x20 || *c > 0x7E {
            return Err("Desc can not use an invisiable ASCII char.");
        }
    }
    Ok(())
}

pub fn is_valid_memo<T: Trait>(msg: &Memo) -> Result {
    // filter char
    // judge len
    if msg.len() as u32 > Module::<T>::memo_len() {
        return Err("memo is too long");
    }
    Ok(())
}

mod imbalances {
    use super::{result, AssetType, ChainT, Imbalance, Saturating, StorageMap, Token, Zero};
    use crate::{Module, TotalAssetBalance, Trait};
    use rstd::mem;

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been created without any equal and opposite accounting.
    #[must_use]
    pub struct PositiveImbalance<T: Trait>(T::Balance, Token, AssetType);
    impl<T: Trait> PositiveImbalance<T> {
        /// Create a new positive imbalance from a balance.
        pub fn new(amount: T::Balance, token: Token, type_: AssetType) -> Self {
            PositiveImbalance(amount, token, type_)
        }
    }

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been destroyed without any equal and opposite accounting.
    #[must_use]
    pub struct NegativeImbalance<T: Trait>(T::Balance, Token, AssetType);
    impl<T: Trait> NegativeImbalance<T> {
        /// Create a new negative imbalance from a balance.
        pub fn new(amount: T::Balance, token: Token, type_: AssetType) -> Self {
            NegativeImbalance(amount, token, type_)
        }
    }

    impl<T: Trait> Imbalance<T::Balance> for PositiveImbalance<T> {
        type Opposite = NegativeImbalance<T>;

        fn zero() -> Self {
            PositiveImbalance::new(Zero::zero(), Module::<T>::TOKEN.to_vec(), AssetType::Free)
        }

        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }

        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;
            // create new object pair
            let r = (
                Self(first, self.1.clone(), self.2),
                Self(second, self.1.clone(), self.2),
            );
            // drop self object
            mem::forget(self);
            r
        }

        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            // drop other object
            mem::forget(other);
            self
        }

        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            // drop other object
            mem::forget(other);
        }

        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            let r = if a >= b {
                Ok(Self::new(a - b, self.1.clone(), self.2))
            } else {
                Err(NegativeImbalance::new(b - a, self.1.clone(), self.2))
            };
            // drop tuple object
            mem::forget((self, other));
            r
        }

        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }

    impl<T: Trait> Imbalance<T::Balance> for NegativeImbalance<T> {
        type Opposite = PositiveImbalance<T>;

        fn zero() -> Self {
            NegativeImbalance::new(Zero::zero(), Module::<T>::TOKEN.to_vec(), AssetType::Free)
        }

        fn drop_zero(self) -> result::Result<(), Self> {
            if self.0.is_zero() {
                Ok(())
            } else {
                Err(self)
            }
        }

        fn split(self, amount: T::Balance) -> (Self, Self) {
            let first = self.0.min(amount);
            let second = self.0 - first;
            // create object pair
            let r = (
                Self(first, self.1.clone(), self.2),
                Self(second, self.1.clone(), self.2),
            );
            // drop self
            mem::forget(self);
            r
        }

        fn merge(mut self, other: Self) -> Self {
            self.0 = self.0.saturating_add(other.0);
            // drop other
            mem::forget(other);
            self
        }

        fn subsume(&mut self, other: Self) {
            self.0 = self.0.saturating_add(other.0);
            // drop other
            mem::forget(other);
        }

        fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
            let (a, b) = (self.0, other.0);
            let r = if a >= b {
                Ok(Self::new(a - b, self.1.clone(), self.2))
            } else {
                Err(PositiveImbalance::new(b - a, self.1.clone(), self.2))
            };
            mem::forget((self, other));
            r
        }

        fn peek(&self) -> T::Balance {
            self.0.clone()
        }
    }

    impl<T: Trait> Drop for PositiveImbalance<T> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            TotalAssetBalance::<T>::mutate(&self.1, |map| {
                let balance = map.entry(self.2).or_default();
                *balance = balance.saturating_add(self.0)
            })
        }
    }

    impl<T: Trait> Drop for NegativeImbalance<T> {
        /// Basic drop handler will just square up the total issuance.
        fn drop(&mut self) {
            TotalAssetBalance::<T>::mutate(&self.1, |map| {
                let balance = map.entry(self.2).or_default();
                *balance = balance.saturating_sub(self.0)
            })
        }
    }

}
