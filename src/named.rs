use core::{cmp::Ordering, fmt, time::Duration};
use num_enum::TryFromPrimitiveError;

#[allow(unused_imports)]
use alloc::string::String;

// When adding a new chain:
//   1. add new variant to the NamedChain enum;
//   2. add extra information in the last `impl` block (explorer URLs, block time) when applicable;
//   3. (optional) add aliases:
//     - Strum (in kebab-case): `#[strum(to_string = "<main>", serialize = "<aliasX>", ...)]`
//      `to_string = "<main>"` must be present and will be used in `Display`, `Serialize`
//      and `FromStr`, while `serialize = "<aliasX>"` will be appended to `FromStr`.
//      More info: <https://docs.rs/strum/latest/strum/additional_attributes/index.html#attributes-on-variants>
//     - Serde (in snake_case): `#[cfg_attr(feature = "serde", serde(alias = "<aliasX>", ...))]`
//      Aliases are appended to the `Deserialize` implementation.
//      More info: <https://serde.rs/variant-attrs.html>
//     - Add a test at the bottom of the file
//   4. run `cargo test --all-features` to update the JSON bindings and schema.

// We don't derive Serialize because it is manually implemented using AsRef<str> and it would break
// a lot of things since Serialize is `kebab-case` vs Deserialize `snake_case`. This means that the
// NamedChain type is not "round-trippable", because the Serialize and Deserialize implementations
// do not use the same case style.

/// An Ethereum EIP-155 chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(strum::IntoStaticStr)] // Into<&'static str>, AsRef<str>, fmt::Display and serde::Serialize
#[derive(strum::VariantNames)] // NamedChain::VARIANTS
#[derive(strum::VariantArray)] // NamedChain::VARIANTS
#[derive(strum::EnumString)] // FromStr, TryFrom<&str>
#[derive(strum::EnumIter)] // NamedChain::iter
#[derive(strum::EnumCount)] // NamedChain::COUNT
#[derive(num_enum::TryFromPrimitive)] // TryFrom<u64>
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[strum(serialize_all = "kebab-case")]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u64)]
#[allow(missing_docs)]
pub enum NamedChain {
    #[strum(to_string = "mainnet", serialize = "ethlive")]
    #[cfg_attr(feature = "serde", serde(alias = "ethlive"))]
    Mainnet = 1,
    Morden = 2,
    Ropsten = 3,
    Rinkeby = 4,
    Goerli = 5,
    Kovan = 42,
    Holesky = 17000,
    Sepolia = 11155111,

    Optimism = 10,
    #[cfg_attr(feature = "serde", serde(alias = "optimism-kovan"))]
    OptimismKovan = 69,
    #[cfg_attr(feature = "serde", serde(alias = "optimism-goerli"))]
    OptimismGoerli = 420,
    #[cfg_attr(feature = "serde", serde(alias = "optimism-sepolia"))]
    OptimismSepolia = 11155420,

    #[cfg_attr(feature = "serde", serde(alias = "arbitrum_one", alias = "arbitrum-one"))]
    Arbitrum = 42161,
    ArbitrumTestnet = 421611,
    #[cfg_attr(feature = "serde", serde(alias = "arbitrum-goerli"))]
    ArbitrumGoerli = 421613,
    #[cfg_attr(feature = "serde", serde(alias = "arbitrum-sepolia"))]
    ArbitrumSepolia = 421614,
    #[cfg_attr(feature = "serde", serde(alias = "arbitrum-nova"))]
    ArbitrumNova = 42170,

    Cronos = 25,
    CronosTestnet = 338,

    Rsk = 30,

    #[strum(to_string = "bsc", serialize = "binance-smart-chain")]
    #[cfg_attr(feature = "serde", serde(alias = "bsc", alias = "binance-smart-chain"))]
    BinanceSmartChain = 56,
    #[strum(to_string = "bsc-testnet", serialize = "binance-smart-chain-testnet")]
    #[cfg_attr(
        feature = "serde",
        serde(alias = "bsc_testnet", alias = "bsc-testnet", alias = "binance-smart-chain-testnet")
    )]
    BinanceSmartChainTestnet = 97,

    Poa = 99,
    Sokol = 77,

    Scroll = 534352,
    #[cfg_attr(
        feature = "serde",
        serde(alias = "scroll_sepolia_testnet", alias = "scroll-sepolia")
    )]
    ScrollSepolia = 534351,

    Metis = 1088,

    #[strum(to_string = "xdai", serialize = "gnosis", serialize = "gnosis-chain")]
    #[cfg_attr(feature = "serde", serde(alias = "xdai", alias = "gnosis", alias = "gnosis-chain"))]
    Gnosis = 100,

    Polygon = 137,
    #[strum(to_string = "mumbai", serialize = "polygon-mumbai")]
    #[cfg_attr(feature = "serde", serde(alias = "mumbai", alias = "polygon-mumbai"))]
    PolygonMumbai = 80001,
    #[strum(to_string = "amoy", serialize = "polygon-amoy")]
    #[cfg_attr(feature = "serde", serde(alias = "amoy", alias = "polygon-amoy"))]
    PolygonAmoy = 80002,
    #[strum(serialize = "polygon-zkevm", serialize = "zkevm")]
    #[cfg_attr(
        feature = "serde",
        serde(alias = "zkevm", alias = "polygon_zkevm", alias = "polygon-zkevm")
    )]
    PolygonZkEvm = 1101,
    #[strum(serialize = "polygon-zkevm-testnet", serialize = "zkevm-testnet")]
    #[cfg_attr(
        feature = "serde",
        serde(
            alias = "zkevm-testnet",
            alias = "polygon_zkevm_testnet",
            alias = "polygon-zkevm-testnet"
        )
    )]
    PolygonZkEvmTestnet = 1442,

    Fantom = 250,
    FantomTestnet = 4002,

    Moonbeam = 1284,
    MoonbeamDev = 1281,

    Moonriver = 1285,

    Moonbase = 1287,

    Dev = 1337,
    #[strum(to_string = "anvil-hardhat", serialize = "anvil", serialize = "hardhat")]
    #[cfg_attr(
        feature = "serde",
        serde(alias = "anvil", alias = "hardhat", alias = "anvil-hardhat")
    )]
    AnvilHardhat = 31337,

    Evmos = 9001,
    EvmosTestnet = 9000,

    Chiado = 10200,

    Oasis = 26863,

    Emerald = 42262,
    EmeraldTestnet = 42261,

    FilecoinMainnet = 314,
    FilecoinCalibrationTestnet = 314159,

    Avalanche = 43114,
    #[strum(to_string = "fuji", serialize = "avalanche-fuji")]
    #[cfg_attr(feature = "serde", serde(alias = "fuji"))]
    AvalancheFuji = 43113,

    Celo = 42220,
    CeloAlfajores = 44787,
    CeloBaklava = 62320,

    Aurora = 1313161554,
    AuroraTestnet = 1313161555,

    Canto = 7700,
    CantoTestnet = 740,

    Boba = 288,

    Base = 8453,
    #[cfg_attr(feature = "serde", serde(alias = "base-goerli"))]
    BaseGoerli = 84531,
    #[cfg_attr(feature = "serde", serde(alias = "base-sepolia"))]
    BaseSepolia = 84532,
    #[cfg_attr(feature = "serde", serde(alias = "syndr"))]
    Syndr = 404,
    #[cfg_attr(feature = "serde", serde(alias = "syndr-sepolia"))]
    SyndrSepolia = 444444,

    Shimmer = 148,

    #[strum(to_string = "fraxtal")]
    #[cfg_attr(feature = "serde", serde(alias = "fraxtal"))]
    Fraxtal = 252,
    #[strum(to_string = "fraxtal-testnet")]
    #[cfg_attr(feature = "serde", serde(alias = "fraxtal-testnet"))]
    FraxtalTestnet = 2522,

    Blast = 81457,
    #[cfg_attr(feature = "serde", serde(alias = "blast-sepolia"))]
    BlastSepolia = 168587773,

    Linea = 59144,
    #[cfg_attr(feature = "serde", serde(alias = "linea-goerli"))]
    LineaGoerli = 59140,

    #[strum(to_string = "zksync")]
    #[cfg_attr(feature = "serde", serde(alias = "zksync"))]
    ZkSync = 324,
    #[strum(to_string = "zksync-testnet")]
    #[cfg_attr(feature = "serde", serde(alias = "zksync_testnet", alias = "zksync-testnet"))]
    ZkSyncTestnet = 280,

    #[strum(to_string = "mantle")]
    #[cfg_attr(feature = "serde", serde(alias = "mantle"))]
    Mantle = 5000,
    #[strum(to_string = "mantle-testnet")]
    #[cfg_attr(feature = "serde", serde(alias = "mantle-testnet"))]
    MantleTestnet = 5001,

    Viction = 88,

    Zora = 7777777,
    #[cfg_attr(feature = "serde", serde(alias = "zora-goerli"))]
    ZoraGoerli = 999,
    #[cfg_attr(feature = "serde", serde(alias = "zora-sepolia"))]
    ZoraSepolia = 999999999,

    Pgn = 424,
    #[cfg_attr(feature = "serde", serde(alias = "pgn-sepolia"))]
    PgnSepolia = 58008,

    Mode = 34443,
    #[cfg_attr(feature = "serde", serde(alias = "mode-sepolia"))]
    ModeSepolia = 919,

    Elastos = 20,

    #[cfg_attr(feature = "serde", serde(alias = "kakarot-sepolia"))]
    KakarotSepolia = 1802203764,

    #[cfg_attr(feature = "serde", serde(alias = "etherlink-testnet"))]
    EtherlinkTestnet = 128123,

    Degen = 666666666,

    #[strum(to_string = "opbnb-mainnet")]
    #[cfg_attr(
        feature = "serde",
        serde(rename = "opbnb_mainnet", alias = "opbnb-mainnet", alias = "op-bnb-mainnet")
    )]
    OpBNBMainnet = 204,
    #[strum(to_string = "opbnb-testnet")]
    #[cfg_attr(
        feature = "serde",
        serde(rename = "opbnb_testnet", alias = "opbnb-testnet", alias = "op-bnb-testnet")
    )]
    OpBNBTestnet = 5611,

    Ronin = 2020,

    Taiko = 167000,
    #[cfg_attr(feature = "serde", serde(alias = "taiko-hekla"))]
    TaikoHekla = 167009,

    #[strum(to_string = "autonomys-nova-testnet")]
    #[cfg_attr(
        feature = "serde",
        serde(rename = "autonomys_nova_testnet", alias = "autonomys-nova-testnet")
    )]
    AutonomysNovaTestnet = 490000,

    Flare = 14,
    #[cfg_attr(feature = "serde", serde(alias = "flare-coston2"))]
    FlareCoston2 = 114,
}

// This must be implemented manually so we avoid a conflict with `TryFromPrimitive` where it treats
// the `#[default]` attribute as its own `#[num_enum(default)]`
impl Default for NamedChain {
    #[inline]
    fn default() -> Self {
        Self::Mainnet
    }
}

macro_rules! impl_into_numeric {
    ($($t:ty)+) => {$(
        impl From<NamedChain> for $t {
            #[inline]
            fn from(chain: NamedChain) -> Self {
                chain as $t
            }
        }
    )+};
}

impl_into_numeric!(u64 i64 u128 i128);
#[cfg(target_pointer_width = "64")]
impl_into_numeric!(usize isize);

macro_rules! impl_try_from_numeric {
    ($($native:ty)+) => {
        $(
            impl TryFrom<$native> for NamedChain {
                type Error = TryFromPrimitiveError<NamedChain>;

                #[inline]
                fn try_from(value: $native) -> Result<Self, Self::Error> {
                    (value as u64).try_into()
                }
            }
        )+
    };
}

impl_try_from_numeric!(u8 i8 u16 i16 u32 i32 usize isize);

impl fmt::Display for NamedChain {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl AsRef<str> for NamedChain {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<u64> for NamedChain {
    #[inline]
    fn eq(&self, other: &u64) -> bool {
        (*self as u64) == *other
    }
}

impl PartialOrd<u64> for NamedChain {
    #[inline]
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {
        (*self as u64).partial_cmp(other)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for NamedChain {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_ref())
    }
}

#[cfg(feature = "rlp")]
impl alloy_rlp::Encodable for NamedChain {
    #[inline]
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        (*self as u64).encode(out)
    }

    #[inline]
    fn length(&self) -> usize {
        (*self as u64).length()
    }
}

#[cfg(feature = "rlp")]
impl alloy_rlp::Decodable for NamedChain {
    #[inline]
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let n = u64::decode(buf)?;
        Self::try_from(n).map_err(|_| alloy_rlp::Error::Overflow)
    }
}

// NB: all utility functions *should* be explicitly exhaustive (not use `_` matcher) so we don't
//     forget to update them when adding a new `NamedChain` variant.
#[allow(clippy::match_like_matches_macro)]
#[deny(unreachable_patterns, unused_variables)]
impl NamedChain {
    /// Returns the string representation of the chain.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    /// Returns the chain's average blocktime, if applicable.
    ///
    /// It can be beneficial to know the average blocktime to adjust the polling of an HTTP provider
    /// for example.
    ///
    /// **Note:** this is not an accurate average, but is rather a sensible default derived from
    /// blocktime charts such as [Etherscan's](https://etherscan.com/chart/blocktime)
    /// or [Polygonscan's](https://polygonscan.com/chart/blocktime).
    ///
    /// # Examples
    ///
    /// ```
    /// use alloy_chains::NamedChain;
    /// use std::time::Duration;
    ///
    /// assert_eq!(NamedChain::Mainnet.average_blocktime_hint(), Some(Duration::from_millis(12_000)),);
    /// assert_eq!(NamedChain::Optimism.average_blocktime_hint(), Some(Duration::from_millis(2_000)),);
    /// ```
    pub const fn average_blocktime_hint(self) -> Option<Duration> {
        use NamedChain as C;

        Some(Duration::from_millis(match self {
            C::Mainnet | C::Taiko | C::TaikoHekla => 12_000,

            C::Arbitrum
            | C::ArbitrumTestnet
            | C::ArbitrumGoerli
            | C::ArbitrumSepolia
            | C::Syndr
            | C::SyndrSepolia
            | C::ArbitrumNova => 260,

            C::Optimism
            | C::OptimismGoerli
            | C::OptimismSepolia
            | C::Base
            | C::BaseGoerli
            | C::BaseSepolia
            | C::Blast
            | C::BlastSepolia
            | C::Fraxtal
            | C::FraxtalTestnet
            | C::Zora
            | C::ZoraGoerli
            | C::ZoraSepolia
            | C::Mode
            | C::ModeSepolia
            | C::Pgn
            | C::PgnSepolia => 2_000,

            C::Viction => 2_000,

            C::Polygon | C::PolygonMumbai | C::PolygonAmoy => 2_100,

            C::Moonbeam | C::Moonriver => 12_500,

            C::BinanceSmartChain | C::BinanceSmartChainTestnet => 3_000,

            C::Avalanche | C::AvalancheFuji => 2_000,

            C::Fantom | C::FantomTestnet => 1_200,

            C::Cronos | C::CronosTestnet | C::Canto | C::CantoTestnet => 5_700,

            C::Evmos | C::EvmosTestnet => 1_900,

            C::Aurora | C::AuroraTestnet => 1_100,

            C::Oasis => 5_500,

            C::Emerald => 6_000,

            C::Dev | C::AnvilHardhat => 200,

            C::Celo | C::CeloAlfajores | C::CeloBaklava => 5_000,

            C::FilecoinCalibrationTestnet | C::FilecoinMainnet => 30_000,

            C::Scroll | C::ScrollSepolia => 3_000,

            C::Shimmer => 5_000,

            C::Gnosis | C::Chiado => 5_000,

            C::Elastos => 5_000,

            C::EtherlinkTestnet => 5_000,

            C::Degen => 600,

            C::Morden
            | C::Ropsten
            | C::Rinkeby
            | C::Goerli
            | C::Kovan
            | C::Sepolia
            | C::Holesky
            | C::Moonbase
            | C::MoonbeamDev
            | C::OptimismKovan
            | C::Poa
            | C::Sokol
            | C::Rsk
            | C::EmeraldTestnet
            | C::Boba
            | C::ZkSync
            | C::ZkSyncTestnet
            | C::PolygonZkEvm
            | C::PolygonZkEvmTestnet
            | C::Metis
            | C::Linea
            | C::LineaGoerli
            | C::Mantle
            | C::MantleTestnet
            | C::KakarotSepolia => return None,

            C::OpBNBMainnet | C::OpBNBTestnet | C::AutonomysNovaTestnet => 1_000,

            C::Ronin => 3_000,

            C::Flare => 1_800,

            C::FlareCoston2 => 2_500,
        }))
    }

    /// Returns whether the chain implements EIP-1559 (with the type 2 EIP-2718 transaction type).
    ///
    /// # Examples
    ///
    /// ```
    /// use alloy_chains::NamedChain;
    ///
    /// assert!(!NamedChain::Mainnet.is_legacy());
    /// assert!(NamedChain::Celo.is_legacy());
    /// ```
    pub const fn is_legacy(self) -> bool {
        use NamedChain as C;

        match self {
            // Known legacy chains / non EIP-1559 compliant.
            C::OptimismKovan
            | C::Fantom
            | C::FantomTestnet
            | C::BinanceSmartChain
            | C::BinanceSmartChainTestnet
            | C::ArbitrumTestnet
            | C::Rsk
            | C::Oasis
            | C::Emerald
            | C::EmeraldTestnet
            | C::Celo
            | C::CeloAlfajores
            | C::CeloBaklava
            | C::Boba
            | C::ZkSync
            | C::ZkSyncTestnet
            | C::Mantle
            | C::MantleTestnet
            | C::PolygonZkEvm
            | C::PolygonZkEvmTestnet
            | C::Scroll
            | C::ScrollSepolia
            | C::Shimmer
            | C::Metis
            | C::Viction
            | C::Elastos
            | C::Ronin => true,

            // Known EIP-1559 chains.
            C::Mainnet
            | C::Goerli
            | C::Sepolia
            | C::Holesky
            | C::Base
            | C::BaseGoerli
            | C::BaseSepolia
            | C::Blast
            | C::BlastSepolia
            | C::Fraxtal
            | C::FraxtalTestnet
            | C::Optimism
            | C::OptimismGoerli
            | C::OptimismSepolia
            | C::Polygon
            | C::PolygonMumbai
            | C::PolygonAmoy
            | C::Avalanche
            | C::AvalancheFuji
            | C::Arbitrum
            | C::ArbitrumGoerli
            | C::ArbitrumSepolia
            | C::ArbitrumNova
            | C::Syndr
            | C::SyndrSepolia
            | C::FilecoinMainnet
            | C::Linea
            | C::LineaGoerli
            | C::FilecoinCalibrationTestnet
            | C::Gnosis
            | C::Chiado
            | C::Zora
            | C::ZoraGoerli
            | C::ZoraSepolia
            | C::Mode
            | C::ModeSepolia
            | C::Pgn
            | C::PgnSepolia
            | C::KakarotSepolia
            | C::EtherlinkTestnet
            | C::Degen
            | C::OpBNBMainnet
            | C::OpBNBTestnet
            | C::Taiko
            | C::TaikoHekla
            | C::AutonomysNovaTestnet
            | C::Flare
            | C::FlareCoston2 => false,

            // Unknown / not applicable, default to false for backwards compatibility.
            C::Dev
            | C::AnvilHardhat
            | C::Morden
            | C::Ropsten
            | C::Rinkeby
            | C::Cronos
            | C::CronosTestnet
            | C::Kovan
            | C::Sokol
            | C::Poa
            | C::Moonbeam
            | C::MoonbeamDev
            | C::Moonriver
            | C::Moonbase
            | C::Evmos
            | C::EvmosTestnet
            | C::Aurora
            | C::AuroraTestnet
            | C::Canto
            | C::CantoTestnet => false,
        }
    }

    /// Returns whether the chain supports the [Shanghai hardfork][ref].
    ///
    /// [ref]: https://github.com/ethereum/execution-specs/blob/master/network-upgrades/mainnet-upgrades/shanghai.md
    pub const fn supports_shanghai(self) -> bool {
        use NamedChain as C;

        match self {
            C::Mainnet
            | C::Goerli
            | C::Sepolia
            | C::Holesky
            | C::Optimism
            | C::OptimismGoerli
            | C::OptimismSepolia
            | C::Base
            | C::BaseGoerli
            | C::BaseSepolia
            | C::Blast
            | C::BlastSepolia
            | C::Fraxtal
            | C::FraxtalTestnet
            | C::Gnosis
            | C::Chiado
            | C::ZoraSepolia
            | C::Mode
            | C::ModeSepolia
            | C::PolygonMumbai
            | C::Polygon
            | C::Arbitrum
            | C::ArbitrumNova
            | C::ArbitrumSepolia
            | C::Syndr
            | C::SyndrSepolia
            | C::EtherlinkTestnet
            | C::Scroll
            | C::ScrollSepolia
            | C::Shimmer
            | C::OpBNBMainnet
            | C::OpBNBTestnet
            | C::KakarotSepolia
            | C::Taiko
            | C::TaikoHekla
            | C::AutonomysNovaTestnet => true,
            _ => false,
        }
    }

    #[doc(hidden)]
    #[deprecated(since = "0.1.3", note = "use `supports_shanghai` instead")]
    pub const fn supports_push0(self) -> bool {
        self.supports_shanghai()
    }

    /// Returns whether the chain is a testnet.
    pub const fn is_testnet(self) -> bool {
        use NamedChain as C;

        match self {
            // Ethereum testnets.
            C::Goerli
            | C::Holesky
            | C::Kovan
            | C::Sepolia
            | C::Morden
            | C::Ropsten
            | C::Rinkeby => true,

            // Other testnets.
            C::ArbitrumGoerli
            | C::ArbitrumSepolia
            | C::ArbitrumTestnet
            | C::SyndrSepolia
            | C::AuroraTestnet
            | C::AvalancheFuji
            | C::BaseGoerli
            | C::BaseSepolia
            | C::BlastSepolia
            | C::BinanceSmartChainTestnet
            | C::CantoTestnet
            | C::CronosTestnet
            | C::CeloAlfajores
            | C::CeloBaklava
            | C::EmeraldTestnet
            | C::EvmosTestnet
            | C::FantomTestnet
            | C::FilecoinCalibrationTestnet
            | C::FraxtalTestnet
            | C::LineaGoerli
            | C::MantleTestnet
            | C::MoonbeamDev
            | C::OptimismGoerli
            | C::OptimismKovan
            | C::OptimismSepolia
            | C::PolygonMumbai
            | C::PolygonAmoy
            | C::PolygonZkEvmTestnet
            | C::ScrollSepolia
            | C::Shimmer
            | C::ZkSyncTestnet
            | C::ZoraGoerli
            | C::ZoraSepolia
            | C::ModeSepolia
            | C::PgnSepolia
            | C::KakarotSepolia
            | C::EtherlinkTestnet
            | C::OpBNBTestnet
            | C::TaikoHekla
            | C::AutonomysNovaTestnet
            | C::FlareCoston2 => true,

            // Dev chains.
            C::Dev | C::AnvilHardhat => true,

            // Mainnets.
            C::Mainnet
            | C::Optimism
            | C::Arbitrum
            | C::ArbitrumNova
            | C::Blast
            | C::Syndr
            | C::Cronos
            | C::Rsk
            | C::BinanceSmartChain
            | C::Poa
            | C::Sokol
            | C::Scroll
            | C::Metis
            | C::Gnosis
            | C::Polygon
            | C::PolygonZkEvm
            | C::Fantom
            | C::Moonbeam
            | C::Moonriver
            | C::Moonbase
            | C::Evmos
            | C::Chiado
            | C::Oasis
            | C::Emerald
            | C::FilecoinMainnet
            | C::Avalanche
            | C::Celo
            | C::Aurora
            | C::Canto
            | C::Boba
            | C::Base
            | C::Fraxtal
            | C::Linea
            | C::ZkSync
            | C::Mantle
            | C::Zora
            | C::Pgn
            | C::Mode
            | C::Viction
            | C::Elastos
            | C::Degen
            | C::OpBNBMainnet
            | C::Ronin
            | C::Taiko
            | C::Flare => false,
        }
    }

    /// Returns the symbol of the chain's native currency.
    pub const fn native_currency_symbol(self) -> Option<&'static str> {
        use NamedChain as C;

        Some(match self {
            C::Mainnet
            | C::Goerli
            | C::Holesky
            | C::Kovan
            | C::Sepolia
            | C::Morden
            | C::Ropsten
            | C::Rinkeby
            | C::Scroll
            | C::ScrollSepolia
            | C::Taiko
            | C::TaikoHekla => "ETH",

            C::BinanceSmartChain
            | C::BinanceSmartChainTestnet
            | C::OpBNBMainnet
            | C::OpBNBTestnet => "BNB",

            C::EtherlinkTestnet => "XTZ",

            C::Degen => "DEGEN",

            C::Ronin => "RON",

            C::Shimmer => "SMR",

            C::Flare => "FLR",

            C::FlareCoston2 => "C2FLR",

            _ => return None,
        })
    }

    /// Returns the chain's blockchain explorer and its API (Etherscan and Etherscan-like) URLs.
    ///
    /// Returns `(API_URL, BASE_URL)`.
    ///
    /// All URLs have no trailing `/`
    ///
    /// # Examples
    ///
    /// ```
    /// use alloy_chains::NamedChain;
    ///
    /// assert_eq!(
    ///     NamedChain::Mainnet.etherscan_urls(),
    ///     Some(("https://api.etherscan.io/api", "https://etherscan.io"))
    /// );
    /// assert_eq!(
    ///     NamedChain::Avalanche.etherscan_urls(),
    ///     Some(("https://api.snowtrace.io/api", "https://snowtrace.io"))
    /// );
    /// assert_eq!(NamedChain::AnvilHardhat.etherscan_urls(), None);
    /// ```
    pub const fn etherscan_urls(self) -> Option<(&'static str, &'static str)> {
        use NamedChain as C;

        Some(match self {
            C::Mainnet => ("https://api.etherscan.io/api", "https://etherscan.io"),
            C::Ropsten => ("https://api-ropsten.etherscan.io/api", "https://ropsten.etherscan.io"),
            C::Kovan => ("https://api-kovan.etherscan.io/api", "https://kovan.etherscan.io"),
            C::Rinkeby => ("https://api-rinkeby.etherscan.io/api", "https://rinkeby.etherscan.io"),
            C::Goerli => ("https://api-goerli.etherscan.io/api", "https://goerli.etherscan.io"),
            C::Sepolia => ("https://api-sepolia.etherscan.io/api", "https://sepolia.etherscan.io"),
            C::Holesky => ("https://api-holesky.etherscan.io/api", "https://holesky.etherscan.io"),

            C::Polygon => ("https://api.polygonscan.com/api", "https://polygonscan.com"),
            C::PolygonMumbai => {
                ("https://api-testnet.polygonscan.com/api", "https://mumbai.polygonscan.com")
            }
            C::PolygonAmoy => {
                ("https://api-amoy.polygonscan.com/api", "https://amoy.polygonscan.com")
            }

            C::PolygonZkEvm => {
                ("https://api-zkevm.polygonscan.com/api", "https://zkevm.polygonscan.com")
            }
            C::PolygonZkEvmTestnet => (
                "https://api-testnet-zkevm.polygonscan.com/api",
                "https://testnet-zkevm.polygonscan.com",
            ),

            C::Avalanche => ("https://api.snowtrace.io/api", "https://snowtrace.io"),
            C::AvalancheFuji => {
                ("https://api-testnet.snowtrace.io/api", "https://testnet.snowtrace.io")
            }

            C::Optimism => {
                ("https://api-optimistic.etherscan.io/api", "https://optimistic.etherscan.io")
            }
            C::OptimismGoerli => (
                "https://api-goerli-optimistic.etherscan.io/api",
                "https://goerli-optimism.etherscan.io",
            ),
            C::OptimismKovan => (
                "https://api-kovan-optimistic.etherscan.io/api",
                "https://kovan-optimistic.etherscan.io",
            ),
            C::OptimismSepolia => (
                "https://api-sepolia-optimistic.etherscan.io/api",
                "https://sepolia-optimism.etherscan.io",
            ),

            C::Fantom => ("https://api.ftmscan.com/api", "https://ftmscan.com"),
            C::FantomTestnet => {
                ("https://api-testnet.ftmscan.com/api", "https://testnet.ftmscan.com")
            }

            C::BinanceSmartChain => ("https://api.bscscan.com/api", "https://bscscan.com"),
            C::BinanceSmartChainTestnet => {
                ("https://api-testnet.bscscan.com/api", "https://testnet.bscscan.com")
            }

            C::OpBNBMainnet => ("https://opbnb.bscscan.com/api", "https://opbnb.bscscan.com"),
            C::OpBNBTestnet => {
                ("https://opbnb-testnet.bscscan.com/api", "https://opbnb-testnet.bscscan.com")
            }

            C::Arbitrum => ("https://api.arbiscan.io/api", "https://arbiscan.io"),
            C::ArbitrumTestnet => {
                ("https://api-testnet.arbiscan.io/api", "https://testnet.arbiscan.io")
            }
            C::ArbitrumGoerli => {
                ("https://api-goerli.arbiscan.io/api", "https://goerli.arbiscan.io")
            }
            C::ArbitrumSepolia => {
                ("https://api-sepolia.arbiscan.io/api", "https://sepolia.arbiscan.io")
            }
            C::ArbitrumNova => ("https://api-nova.arbiscan.io/api", "https://nova.arbiscan.io"),

            C::Syndr => ("https://explorer.syndr.com/api", "https://explorer.syndr.com"),
            C::SyndrSepolia => {
                ("https://sepolia-explorer.syndr.com/api", "https://sepolia-explorer.syndr.com")
            }

            C::Cronos => ("https://api.cronoscan.com/api", "https://cronoscan.com"),
            C::CronosTestnet => {
                ("https://api-testnet.cronoscan.com/api", "https://testnet.cronoscan.com")
            }

            C::Moonbeam => ("https://api-moonbeam.moonscan.io/api", "https://moonbeam.moonscan.io"),
            C::Moonbase => ("https://api-moonbase.moonscan.io/api", "https://moonbase.moonscan.io"),
            C::Moonriver => {
                ("https://api-moonriver.moonscan.io/api", "https://moonriver.moonscan.io")
            }

            C::Gnosis => ("https://api.gnosisscan.io/api", "https://gnosisscan.io"),

            C::Scroll => ("https://api.scrollscan.com/api", "https://scrollscan.com"),
            C::ScrollSepolia => {
                ("https://api-sepolia.scrollscan.com/api", "https://sepolia.scrollscan.com")
            }

            C::Shimmer => {
                ("https://explorer.evm.shimmer.network/api", "https://explorer.evm.shimmer.network")
            }

            C::Metis => {
                ("https://andromeda-explorer.metis.io/api", "https://andromeda-explorer.metis.io")
            }

            C::Chiado => {
                ("https://blockscout.chiadochain.net/api", "https://blockscout.chiadochain.net")
            }

            C::FilecoinCalibrationTestnet => (
                "https://api.calibration.node.glif.io/rpc/v1",
                "https://calibration.filfox.info/en",
            ),

            C::Sokol => {
                ("https://blockscout.com/poa/sokol/api", "https://blockscout.com/poa/sokol")
            }

            C::Poa => ("https://blockscout.com/poa/core/api", "https://blockscout.com/poa/core"),

            C::Rsk => {
                ("https://blockscout.com/rsk/mainnet/api", "https://blockscout.com/rsk/mainnet")
            }

            C::Oasis => ("https://scan.oasischain.io/api", "https://scan.oasischain.io"),

            C::Emerald => {
                ("https://explorer.emerald.oasis.dev/api", "https://explorer.emerald.oasis.dev")
            }
            C::EmeraldTestnet => (
                "https://testnet.explorer.emerald.oasis.dev/api",
                "https://testnet.explorer.emerald.oasis.dev",
            ),

            C::Aurora => ("https://api.aurorascan.dev/api", "https://aurorascan.dev"),
            C::AuroraTestnet => {
                ("https://testnet.aurorascan.dev/api", "https://testnet.aurorascan.dev")
            }

            C::Evmos => ("https://evm.evmos.org/api", "https://evm.evmos.org"),
            C::EvmosTestnet => ("https://evm.evmos.dev/api", "https://evm.evmos.dev"),

            C::Celo => {
                ("https://explorer.celo.org/mainnet/api", "https://explorer.celo.org/mainnet")
            }
            C::CeloAlfajores => {
                ("https://explorer.celo.org/alfajores/api", "https://explorer.celo.org/alfajores")
            }
            C::CeloBaklava => {
                ("https://explorer.celo.org/baklava/api", "https://explorer.celo.org/baklava")
            }

            C::Canto => ("https://evm.explorer.canto.io/api", "https://evm.explorer.canto.io"),
            C::CantoTestnet => (
                "https://testnet-explorer.canto.neobase.one/api",
                "https://testnet-explorer.canto.neobase.one",
            ),

            C::Boba => ("https://api.bobascan.com/api", "https://bobascan.com"),

            C::Base => ("https://api.basescan.org/api", "https://basescan.org"),
            C::BaseGoerli => ("https://api-goerli.basescan.org/api", "https://goerli.basescan.org"),
            C::BaseSepolia => {
                ("https://api-sepolia.basescan.org/api", "https://sepolia.basescan.org")
            }

            C::Fraxtal => ("https://api.fraxscan.com/api", "https://fraxscan.com"),
            C::FraxtalTestnet => {
                ("https://api-holesky.fraxscan.com/api", "https://holesky.fraxscan.com")
            }

            C::Blast => ("https://api.blastscan.io/api", "https://blastscan.io"),
            C::BlastSepolia => {
                ("https://api-sepolia.blastscan.io/api", "https://sepolia.blastscan.io")
            }

            C::ZkSync => {
                ("https://zksync2-mainnet-explorer.zksync.io", "https://explorer.zksync.io")
            }
            C::ZkSyncTestnet => {
                ("https://zksync2-testnet-explorer.zksync.dev", "https://goerli.explorer.zksync.io")
            }

            C::Linea => ("https://api.lineascan.build/api", "https://lineascan.build"),
            C::LineaGoerli => {
                ("https://explorer.goerli.linea.build/api", "https://explorer.goerli.linea.build")
            }

            C::Mantle => ("https://explorer.mantle.xyz/api", "https://explorer.mantle.xyz"),
            C::MantleTestnet => {
                ("https://explorer.testnet.mantle.xyz/api", "https://explorer.testnet.mantle.xyz")
            }

            C::Viction => ("https://www.vicscan.xyz/api", "https://www.vicscan.xyz"),

            C::Zora => ("https://explorer.zora.energy/api", "https://explorer.zora.energy"),
            C::ZoraGoerli => {
                ("https://testnet.explorer.zora.energy/api", "https://testnet.explorer.zora.energy")
            }
            C::ZoraSepolia => {
                ("https://sepolia.explorer.zora.energy/api", "https://sepolia.explorer.zora.energy")
            }

            C::Pgn => {
                ("https://explorer.publicgoods.network/api", "https://explorer.publicgoods.network")
            }

            C::PgnSepolia => (
                "https://explorer.sepolia.publicgoods.network/api",
                "https://explorer.sepolia.publicgoods.network",
            ),

            C::Mode => ("https://explorer.mode.network/api", "https://explorer.mode.network"),
            C::ModeSepolia => (
                "https://sepolia.explorer.mode.network/api",
                "https://sepolia.explorer.mode.network",
            ),

            C::Elastos => ("https://esc.elastos.io/api", "https://esc.elastos.io"),

            C::AnvilHardhat
            | C::Dev
            | C::Morden
            | C::MoonbeamDev
            | C::FilecoinMainnet
            | C::AutonomysNovaTestnet => {
                return None;
            }
            C::KakarotSepolia => {
                ("https://sepolia.kakarotscan.org/api", "https://sepolia.kakarotscan.org")
            }
            C::EtherlinkTestnet => (
                "https://testnet-explorer.etherlink.com/api",
                "https://testnet-explorer.etherlink.com",
            ),
            C::Degen => ("https://explorer.degen.tips/api", "https://explorer.degen.tips"),
            C::Ronin => ("https://skynet-api.roninchain.com/ronin", "https://app.roninchain.com"),
            C::Taiko => ("https://api.taikoscan.io/api", "https://taikoscan.io"),
            C::TaikoHekla => ("https://api-testnet.taikoscan.io/api", "https://hekla.taikoscan.io"),
            C::Flare => {
                ("https://flare-explorer.flare.network/api", "https://flare-explorer.flare.network")
            }
            C::FlareCoston2 => (
                "https://coston2-explorer.flare.network/api",
                "https://coston2-explorer.flare.network",
            ),
        })
    }

    /// Returns the chain's blockchain explorer's API key environment variable's default name.
    ///
    /// # Examples
    ///
    /// ```
    /// use alloy_chains::NamedChain;
    ///
    /// assert_eq!(NamedChain::Mainnet.etherscan_api_key_name(), Some("ETHERSCAN_API_KEY"));
    /// assert_eq!(NamedChain::AnvilHardhat.etherscan_api_key_name(), None);
    /// ```
    pub const fn etherscan_api_key_name(self) -> Option<&'static str> {
        use NamedChain as C;

        let api_key_name = match self {
            C::Mainnet
            | C::Morden
            | C::Ropsten
            | C::Kovan
            | C::Rinkeby
            | C::Goerli
            | C::Holesky
            | C::Optimism
            | C::OptimismGoerli
            | C::OptimismKovan
            | C::OptimismSepolia
            | C::BinanceSmartChain
            | C::BinanceSmartChainTestnet
            | C::OpBNBMainnet
            | C::OpBNBTestnet
            | C::Arbitrum
            | C::ArbitrumTestnet
            | C::ArbitrumGoerli
            | C::ArbitrumSepolia
            | C::ArbitrumNova
            | C::Syndr
            | C::SyndrSepolia
            | C::Cronos
            | C::CronosTestnet
            | C::Aurora
            | C::AuroraTestnet
            | C::Celo
            | C::CeloAlfajores
            | C::CeloBaklava
            | C::Base
            | C::Linea
            | C::Mantle
            | C::MantleTestnet
            | C::BaseGoerli
            | C::BaseSepolia
            | C::Fraxtal
            | C::FraxtalTestnet
            | C::Blast
            | C::BlastSepolia
            | C::Gnosis
            | C::Scroll
            | C::ScrollSepolia
            | C::Taiko
            | C::TaikoHekla => "ETHERSCAN_API_KEY",

            C::Avalanche | C::AvalancheFuji => "SNOWTRACE_API_KEY",

            C::Polygon
            | C::PolygonMumbai
            | C::PolygonAmoy
            | C::PolygonZkEvm
            | C::PolygonZkEvmTestnet => "POLYGONSCAN_API_KEY",

            C::Fantom | C::FantomTestnet => "FTMSCAN_API_KEY",

            C::Moonbeam | C::Moonbase | C::MoonbeamDev | C::Moonriver => "MOONSCAN_API_KEY",

            C::Canto
            | C::CantoTestnet
            | C::Zora
            | C::ZoraGoerli
            | C::ZoraSepolia
            | C::Mode
            | C::ModeSepolia
            | C::Pgn
            | C::PgnSepolia
            | C::KakarotSepolia
            | C::EtherlinkTestnet
            | C::Shimmer
            | C::Flare
            | C::FlareCoston2 => "BLOCKSCOUT_API_KEY",

            C::Boba => "BOBASCAN_API_KEY",

            // Explicitly exhaustive. See NB above.
            C::Metis
            | C::Chiado
            | C::Sepolia
            | C::Rsk
            | C::Sokol
            | C::Poa
            | C::Oasis
            | C::Emerald
            | C::EmeraldTestnet
            | C::Evmos
            | C::EvmosTestnet
            | C::AnvilHardhat
            | C::Dev
            | C::ZkSync
            | C::ZkSyncTestnet
            | C::FilecoinMainnet
            | C::LineaGoerli
            | C::FilecoinCalibrationTestnet
            | C::Viction
            | C::Elastos
            | C::Degen
            | C::Ronin
            | C::AutonomysNovaTestnet => return None,
        };

        Some(api_key_name)
    }

    /// Returns the chain's blockchain explorer's API key, from the environment variable with the
    /// name specified in [`etherscan_api_key_name`](NamedChain::etherscan_api_key_name).
    ///
    /// # Examples
    ///
    /// ```
    /// use alloy_chains::NamedChain;
    ///
    /// let chain = NamedChain::Mainnet;
    /// std::env::set_var(chain.etherscan_api_key_name().unwrap(), "KEY");
    /// assert_eq!(chain.etherscan_api_key().as_deref(), Some("KEY"));
    /// ```
    #[cfg(feature = "std")]
    pub fn etherscan_api_key(self) -> Option<String> {
        self.etherscan_api_key_name().and_then(|name| std::env::var(name).ok())
    }

    /// Returns the address of the public DNS node list for the given chain.
    ///
    /// See also <https://github.com/ethereum/discv4-dns-lists>.
    pub fn public_dns_network_protocol(self) -> Option<String> {
        use NamedChain as C;

        const DNS_PREFIX: &str = "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@";
        if let C::Mainnet | C::Goerli | C::Sepolia | C::Ropsten | C::Rinkeby | C::Holesky = self {
            // `{DNS_PREFIX}all.{self.lower()}.ethdisco.net`
            let mut s = String::with_capacity(DNS_PREFIX.len() + 32);
            s.push_str(DNS_PREFIX);
            s.push_str("all.");
            let chain_str = self.as_ref();
            s.push_str(chain_str);
            let l = s.len();
            s[l - chain_str.len()..].make_ascii_lowercase();
            s.push_str(".ethdisco.net");

            Some(s)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::{EnumCount, IntoEnumIterator};

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
    fn aliases() {
        use NamedChain::*;

        // kebab-case
        const ALIASES: &[(NamedChain, &[&str])] = &[
            (Mainnet, &["ethlive"]),
            (BinanceSmartChain, &["bsc", "binance-smart-chain"]),
            (BinanceSmartChainTestnet, &["bsc-testnet", "binance-smart-chain-testnet"]),
            (Gnosis, &["gnosis", "gnosis-chain"]),
            (PolygonMumbai, &["mumbai"]),
            (PolygonZkEvm, &["zkevm", "polygon-zkevm"]),
            (PolygonZkEvmTestnet, &["zkevm-testnet", "polygon-zkevm-testnet"]),
            (AnvilHardhat, &["anvil", "hardhat"]),
            (AvalancheFuji, &["fuji"]),
            (ZkSync, &["zksync"]),
            (Mantle, &["mantle"]),
            (MantleTestnet, &["mantle-testnet"]),
            (Base, &["base"]),
            (BaseGoerli, &["base-goerli"]),
            (BaseSepolia, &["base-sepolia"]),
            (Fraxtal, &["fraxtal"]),
            (FraxtalTestnet, &["fraxtal-testnet"]),
            (BlastSepolia, &["blast-sepolia"]),
            (Syndr, &["syndr"]),
            (SyndrSepolia, &["syndr-sepolia"]),
            (LineaGoerli, &["linea-goerli"]),
            (AutonomysNovaTestnet, &["autonomys-nova-testnet"]),
        ];

        for &(chain, aliases) in ALIASES {
            for &alias in aliases {
                let named = alias.parse::<NamedChain>().unwrap();
                assert_eq!(named, chain);

                #[cfg(feature = "serde")]
                {
                    assert_eq!(
                        serde_json::from_str::<NamedChain>(&format!("\"{alias}\"")).unwrap(),
                        chain
                    );

                    assert_eq!(
                        serde_json::from_str::<NamedChain>(&format!("\"{named}\"")).unwrap(),
                        chain
                    );
                }
            }
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
                assert!(!api.ends_with('/'), "{:?} api url has trailing /", chain);
                assert!(!base.ends_with('/'), "{:?} base url has trailing /", chain);
            }
        }
    }
}
