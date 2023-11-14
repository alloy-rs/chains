use crate::NamedChain;
use alloc::string::String;
use core::{fmt, str::FromStr};

#[cfg(feature = "arbitrary")]
use proptest::{
    sample::Selector,
    strategy::{Map, TupleUnion, WA},
};
#[cfg(feature = "arbitrary")]
use strum::{EnumCount, IntoEnumIterator};

/// Either a known [`NamedChain`] or a EIP-155 chain ID.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chain(ChainKind);

/// The kind of chain. Returned by [`Chain::kind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChainKind {
    /// Known chain.
    Named(NamedChain),
    /// EIP-155 chain ID.
    Id(u64),
}

impl fmt::Debug for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Chain::")?;
        self.kind().fmt(f)
    }
}

impl Default for Chain {
    #[inline]
    fn default() -> Self {
        Self::from_named(NamedChain::Mainnet)
    }
}

impl From<NamedChain> for Chain {
    #[inline]
    fn from(id: NamedChain) -> Self {
        Self::from_named(id)
    }
}

impl From<u64> for Chain {
    #[inline]
    fn from(id: u64) -> Self {
        Self::from_id(id)
    }
}

impl From<Chain> for u64 {
    #[inline]
    fn from(chain: Chain) -> Self {
        chain.id()
    }
}

impl TryFrom<Chain> for NamedChain {
    type Error = <NamedChain as TryFrom<u64>>::Error;

    #[inline]
    fn try_from(chain: Chain) -> Result<Self, Self::Error> {
        match *chain.kind() {
            ChainKind::Named(chain) => Ok(chain),
            ChainKind::Id(id) => id.try_into(),
        }
    }
}

impl FromStr for Chain {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(chain) = NamedChain::from_str(s) {
            Ok(Self::from_named(chain))
        } else {
            s.parse::<u64>().map(Self::from_id)
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            ChainKind::Named(chain) => chain.fmt(f),
            ChainKind::Id(id) => id.fmt(f),
        }
    }
}

#[cfg(feature = "rlp")]
impl alloy_rlp::Encodable for Chain {
    #[inline]
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.id().encode(out)
    }

    #[inline]
    fn length(&self) -> usize {
        self.id().length()
    }
}

#[cfg(feature = "rlp")]
impl alloy_rlp::Decodable for Chain {
    #[inline]
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        u64::decode(buf).map(Self::from)
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Chain {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.ratio(1, 2)? {
            let chain = u.int_in_range(0..=(NamedChain::COUNT - 1))?;

            return Ok(Self::from_named(NamedChain::iter().nth(chain).expect("in range")));
        }

        Ok(Self::from_id(u64::arbitrary(u)?))
    }
}

#[cfg(feature = "arbitrary")]
impl proptest::arbitrary::Arbitrary for Chain {
    type Parameters = ();
    type Strategy = TupleUnion<(
        WA<Map<proptest::sample::SelectorStrategy, fn(proptest::sample::Selector) -> Chain>>,
        WA<Map<proptest::num::u64::Any, fn(u64) -> Chain>>,
    )>;

    fn arbitrary_with((): ()) -> Self::Strategy {
        use proptest::prelude::*;
        prop_oneof![
            any::<Selector>().prop_map(move |sel| Self::from_named(sel.select(NamedChain::iter()))),
            any::<u64>().prop_map(Self::from_id),
        ]
    }
}

impl Chain {
    #[allow(non_snake_case)]
    #[doc(hidden)]
    #[deprecated(since = "0.1.0", note = "use `Self::from_named()` instead")]
    #[inline]
    pub const fn Named(named: NamedChain) -> Self {
        Self::from_named(named)
    }

    #[allow(non_snake_case)]
    #[doc(hidden)]
    #[deprecated(since = "0.1.0", note = "use `Self::from_id()` instead")]
    #[inline]
    pub const fn Id(id: u64) -> Self {
        Self::from_id_unchecked(id)
    }

    /// Creates a new [`Chain`] by wrapping a [`NamedChain`].
    #[inline]
    pub const fn from_named(named: NamedChain) -> Self {
        Self(ChainKind::Named(named))
    }

    /// Creates a new [`Chain`] by wrapping a [`NamedChain`].
    #[inline]
    pub fn from_id(id: u64) -> Self {
        if let Ok(named) = NamedChain::try_from(id) {
            Self::from_named(named)
        } else {
            Self::from_id_unchecked(id)
        }
    }

    /// Creates a new [`Chain`] from the given ID, without checking if an associated [`NamedChain`]
    /// exists.
    ///
    /// This is discouraged, as other methods assume that the chain ID is not known, but it is not
    /// unsafe.
    #[inline]
    pub const fn from_id_unchecked(id: u64) -> Self {
        Self(ChainKind::Id(id))
    }

    /// Returns the mainnet chain.
    #[inline]
    pub const fn mainnet() -> Self {
        Self::from_named(NamedChain::Mainnet)
    }

    /// Returns the goerli chain.
    #[inline]
    pub const fn goerli() -> Self {
        Self::from_named(NamedChain::Goerli)
    }

    /// Returns the sepolia chain.
    #[inline]
    pub const fn sepolia() -> Self {
        Self::from_named(NamedChain::Sepolia)
    }

    /// Returns the holesky chain.
    #[inline]
    pub const fn holesky() -> Self {
        Self::from_named(NamedChain::Holesky)
    }

    /// Returns the optimism goerli chain.
    #[inline]
    pub const fn optimism_goerli() -> Self {
        Self::from_named(NamedChain::OptimismGoerli)
    }

    /// Returns the optimism mainnet chain.
    #[inline]
    pub const fn optimism_mainnet() -> Self {
        Self::from_named(NamedChain::Optimism)
    }

    /// Returns the base goerli chain.
    #[inline]
    pub const fn base_goerli() -> Self {
        Self::from_named(NamedChain::BaseGoerli)
    }

    /// Returns the base mainnet chain.
    #[inline]
    pub const fn base_mainnet() -> Self {
        Self::from_named(NamedChain::Base)
    }

    /// Returns the dev chain.
    #[inline]
    pub const fn dev() -> Self {
        Self::from_named(NamedChain::Dev)
    }

    /// Returns the kind of this chain.
    #[inline]
    pub const fn kind(&self) -> &ChainKind {
        &self.0
    }

    /// Returns the kind of this chain.
    #[inline]
    pub const fn into_kind(self) -> ChainKind {
        self.0
    }

    /// Returns true if the chain contains Optimism configuration.
    #[inline]
    pub const fn is_optimism(self) -> bool {
        matches!(
            self.kind(),
            ChainKind::Named(
                NamedChain::Optimism
                    | NamedChain::OptimismGoerli
                    | NamedChain::OptimismKovan
                    | NamedChain::OptimismSepolia
                    | NamedChain::Base
                    | NamedChain::BaseGoerli
            )
        )
    }

    /// Attempts to convert the chain into a named chain.
    #[inline]
    pub const fn named(self) -> Option<NamedChain> {
        match *self.kind() {
            ChainKind::Named(named) => Some(named),
            ChainKind::Id(_) => None,
        }
    }

    /// The ID of the chain.
    #[inline]
    pub const fn id(self) -> u64 {
        match *self.kind() {
            ChainKind::Named(named) => named as u64,
            ChainKind::Id(id) => id,
        }
    }

    /// Returns the address of the public DNS node list for the given chain.
    ///
    /// See also <https://github.com/ethereum/discv4-dns-lists>
    pub fn public_dns_network_protocol(self) -> Option<String> {
        use NamedChain as C;
        const DNS_PREFIX: &str = "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@";

        let named: NamedChain = self.try_into().ok()?;
        if matches!(named, C::Mainnet | C::Goerli | C::Sepolia | C::Ropsten | C::Rinkeby) {
            return Some(format!("{DNS_PREFIX}all.{}.ethdisco.net", named.as_ref().to_lowercase()));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_id() {
        assert_eq!(Chain::from_id(1234).id(), 1234);
    }

    #[test]
    fn test_named_id() {
        assert_eq!(Chain::from_named(NamedChain::Goerli).id(), 5);
    }

    #[test]
    fn test_display_named_chain() {
        assert_eq!(Chain::from_named(NamedChain::Mainnet).to_string(), "mainnet");
    }

    #[test]
    fn test_display_id_chain() {
        assert_eq!(Chain::from_id(1234).to_string(), "1234");
    }

    #[test]
    fn test_from_str_named_chain() {
        let result = Chain::from_str("mainnet");
        let expected = Chain::from_named(NamedChain::Mainnet);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_from_str_named_chain_error() {
        let result = Chain::from_str("chain");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_id_chain() {
        let result = Chain::from_str("1234");
        let expected = Chain::from_id(1234);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_default() {
        let default = Chain::default();
        let expected = Chain::from_named(NamedChain::Mainnet);
        assert_eq!(default, expected);
    }

    #[cfg(feature = "rlp")]
    #[test]
    fn test_id_chain_encodable_length() {
        use alloy_rlp::Encodable;

        let chain = Chain::from_id(1234);
        assert_eq!(chain.length(), 3);
    }

    #[test]
    fn test_dns_network() {
        let s = "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@all.mainnet.ethdisco.net";
        let chain: Chain = NamedChain::Mainnet.into();
        assert_eq!(s, chain.public_dns_network_protocol().unwrap().as_str());
    }
}
