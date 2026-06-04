pub use crate::generated::named::{NamedChain, NamedChainIter, TryFromChainIdError};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::named::PARSE_ALIASES;
    #[cfg(feature = "serde")]
    use crate::generated::named::SERDE_ALIASES;

    #[allow(unused_imports)]
    use alloc::string::ToString;

    #[test]
    #[cfg(feature = "serde")]
    fn default() {
        assert_eq!(serde_json::to_string(&NamedChain::default()).unwrap(), "\"mainnet\"");
    }

    #[test]
    fn enum_iter() {
        assert_eq!(NamedChain::COUNT, NamedChain::iter().size_hint().0);
        assert_eq!(NamedChain::COUNT, <NamedChain as strum::EnumCount>::COUNT);
        assert_eq!(NamedChain::VARIANTS, <NamedChain as strum::VariantArray>::VARIANTS);
        assert_eq!(NamedChain::VARIANT_NAMES, <NamedChain as strum::VariantNames>::VARIANTS);
    }

    #[test]
    fn roundtrip_string() {
        for chain in NamedChain::iter() {
            let chain_string = chain.to_string();
            assert_eq!(chain_string, format!("{chain}"));
            assert_eq!(chain_string.as_str(), chain.as_ref());
            #[cfg(feature = "serde")]
            assert_eq!(serde_json::to_string(&chain).unwrap(), format!("\"{chain_string}\""));

            assert_eq!(chain_string.parse::<NamedChain>().unwrap(), chain);
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn roundtrip_serde() {
        for chain in NamedChain::iter() {
            let chain_string = serde_json::to_string(&chain).unwrap();
            let chain_string = chain_string.replace('-', "_");
            assert_eq!(serde_json::from_str::<'_, NamedChain>(&chain_string).unwrap(), chain);
        }
    }

    #[test]
    #[cfg(feature = "arbitrary")]
    fn test_arbitrary_named_chain() {
        use arbitrary::{Arbitrary, Unstructured};
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 255];
        let mut unstructured = Unstructured::new(&data);

        for _ in 0..10 {
            let _chain = NamedChain::arbitrary(&mut unstructured).unwrap();
        }
    }

    #[test]
    fn aliases() {
        for &(chain, alias) in PARSE_ALIASES {
            assert_eq!(alias.parse::<NamedChain>().unwrap(), chain);

            #[cfg(feature = "serde")]
            assert_eq!(serde_json::from_str::<NamedChain>(&format!("\"{alias}\"")).unwrap(), chain);
        }

        #[cfg(feature = "serde")]
        for &(chain, alias) in SERDE_ALIASES {
            assert_eq!(serde_json::from_str::<NamedChain>(&format!("\"{alias}\"")).unwrap(), chain);
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_to_string_match() {
        for chain in NamedChain::iter() {
            let chain_serde = serde_json::to_string(&chain).unwrap();
            let chain_string = format!("\"{chain}\"");
            assert_eq!(chain_serde, chain_string);
        }
    }

    #[test]
    fn test_dns_network() {
        let s = "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@all.mainnet.ethdisco.net";
        assert_eq!(NamedChain::Mainnet.public_dns_network_protocol().unwrap(), s);
    }

    #[test]
    fn ensure_no_trailing_etherscan_url_separator() {
        for chain in NamedChain::iter() {
            if let Some((api, base)) = chain.etherscan_urls() {
                assert!(!api.ends_with('/'), "{chain:?} api url has trailing /");
                assert!(!base.ends_with('/'), "{chain:?} base url has trailing /");
            }
        }
    }
}
