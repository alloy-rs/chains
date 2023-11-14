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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Chain {
    /// Known chain.
    Named(NamedChain),
    /// EIP-155 chain ID.
    Id(u64),
}

impl Chain {
    /// Returns the mainnet chain.
    pub const fn mainnet() -> Self {
        Chain::Named(NamedChain::Mainnet)
    }

    /// Returns the goerli chain.
    pub const fn goerli() -> Self {
        Chain::Named(NamedChain::Goerli)
    }

    /// Returns the sepolia chain.
    pub const fn sepolia() -> Self {
        Chain::Named(NamedChain::Sepolia)
    }

    /// Returns the holesky chain.
    pub const fn holesky() -> Self {
        Chain::Named(NamedChain::Holesky)
    }

    /// Returns the optimism goerli chain.
    pub const fn optimism_goerli() -> Self {
        Chain::Named(NamedChain::OptimismGoerli)
    }

    /// Returns the optimism mainnet chain.
    pub const fn optimism_mainnet() -> Self {
        Chain::Named(NamedChain::Optimism)
    }

    /// Returns the base goerli chain.
    pub const fn base_goerli() -> Self {
        Chain::Named(NamedChain::BaseGoerli)
    }

    /// Returns the base mainnet chain.
    pub const fn base_mainnet() -> Self {
        Chain::Named(NamedChain::Base)
    }

    /// Returns the dev chain.
    pub const fn dev() -> Self {
        Chain::Named(NamedChain::Dev)
    }

    /// Returns true if the chain contains Optimism configuration.
    pub fn is_optimism(self) -> bool {
        self.named().map_or(false, |c| {
            matches!(
                c,
                NamedChain::Optimism
                    | NamedChain::OptimismGoerli
                    | NamedChain::OptimismKovan
                    | NamedChain::Base
                    | NamedChain::BaseGoerli
            )
        })
    }

    /// Attempts to convert the chain into a named chain.
    pub fn named(&self) -> Option<NamedChain> {
        match self {
            Chain::Named(chain) => Some(*chain),
            Chain::Id(id) => NamedChain::try_from(*id).ok(),
        }
    }

    /// The ID of the chain.
    pub const fn id(&self) -> u64 {
        match self {
            Chain::Named(chain) => *chain as u64,
            Chain::Id(id) => *id,
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

impl Default for Chain {
    fn default() -> Self {
        NamedChain::Mainnet.into()
    }
}

impl From<NamedChain> for Chain {
    fn from(id: NamedChain) -> Self {
        Chain::Named(id)
    }
}

impl From<u64> for Chain {
    fn from(id: u64) -> Self {
        NamedChain::try_from(id).map(Chain::Named).unwrap_or_else(|_| Chain::Id(id))
    }
}

impl From<Chain> for u64 {
    fn from(c: Chain) -> Self {
        match c {
            Chain::Named(c) => c as u64,
            Chain::Id(id) => id,
        }
    }
}

impl TryFrom<Chain> for NamedChain {
    type Error = <NamedChain as TryFrom<u64>>::Error;

    fn try_from(chain: Chain) -> Result<Self, Self::Error> {
        match chain {
            Chain::Named(chain) => Ok(chain),
            Chain::Id(id) => id.try_into(),
        }
    }
}

impl FromStr for Chain {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(chain) = NamedChain::from_str(s) {
            Ok(Chain::Named(chain))
        } else {
            s.parse::<u64>().map(Chain::Id)
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chain::Named(chain) => chain.fmt(f),
            Chain::Id(id) => {
                if let Ok(chain) = NamedChain::try_from(*id) {
                    chain.fmt(f)
                } else {
                    id.fmt(f)
                }
            }
        }
    }
}

#[cfg(TODO)]
impl alloy_rlp::Encodable for Chain {
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        match self {
            Self::Named(chain) => u64::from(*chain).encode(out),
            Self::Id(id) => id.encode(out),
        }
    }

    fn length(&self) -> usize {
        match self {
            Self::Named(chain) => u64::from(*chain).length(),
            Self::Id(id) => id.length(),
        }
    }
}

#[cfg(TODO)]
impl alloy_rlp::Decodable for Chain {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(u64::decode(buf)?.into())
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Chain {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.ratio(1, 2)? {
            let chain = u.int_in_range(0..=(NamedChain::COUNT - 1))?;

            return Ok(Chain::Named(NamedChain::iter().nth(chain).expect("in range")));
        }

        Ok(Self::Id(u64::arbitrary(u)?))
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
            any::<Selector>().prop_map(move |sel| Chain::Named(sel.select(NamedChain::iter()))),
            any::<u64>().prop_map(Chain::Id),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id() {
        let chain = Chain::Id(1234);
        assert_eq!(chain.id(), 1234);
    }

    #[test]
    fn test_named_id() {
        let chain = Chain::Named(NamedChain::Goerli);
        assert_eq!(chain.id(), 5);
    }

    #[test]
    fn test_display_named_chain() {
        let chain = Chain::Named(NamedChain::Mainnet);
        assert_eq!(format!("{chain}"), "mainnet");
    }

    #[test]
    fn test_display_id_chain() {
        let chain = Chain::Id(1234);
        assert_eq!(format!("{chain}"), "1234");
    }

    #[test]
    fn test_from_str_named_chain() {
        let result = Chain::from_str("mainnet");
        let expected = Chain::Named(NamedChain::Mainnet);

        assert!(result.is_ok());
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
        let expected = Chain::Id(1234);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_default() {
        let default = Chain::default();
        let expected = Chain::Named(NamedChain::Mainnet);

        assert_eq!(default, expected);
    }

    #[cfg(TODO)]
    #[test]
    fn test_id_chain_encodable_length() {
        let chain = Chain::Id(1234);

        assert_eq!(chain.length(), 3);
    }

    #[test]
    fn test_dns_network() {
        let s = "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@all.mainnet.ethdisco.net";
        let chain: Chain = NamedChain::Mainnet.into();
        assert_eq!(s, chain.public_dns_network_protocol().unwrap().as_str());
    }
}
