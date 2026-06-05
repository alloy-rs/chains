#!/usr/bin/env python3
"""Update generated chain registry artifacts."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import urllib.request
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CHAINLIST_URL = "https://chainid.network/chains.json"
MANUAL_PATH = ROOT / "registry" / "manual.json"
ASSET_CHAINS_PATH = ROOT / "assets" / "chains.json"
GENERATED_MOD_PATH = ROOT / "src" / "generated" / "mod.rs"
GENERATED_NAMED_PATH = ROOT / "src" / "generated" / "named.rs"


@dataclass(frozen=True)
class Chain:
    chain_id: int
    internal_id: str
    name: str
    aliases: tuple[str, ...]
    serde_name: str | None
    serde_aliases: tuple[str, ...]
    average_blocktime_hint: int | None
    is_legacy: bool
    supports_shanghai: bool
    is_testnet: bool
    native_currency_symbol: str | None
    etherscan_api_url: str | None
    etherscan_base_url: str | None
    etherscan_api_key_name: str | None
    tags: frozenset[str]
    wrapped_native_token: str | None
    manual_only: bool


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manual", type=Path, default=MANUAL_PATH)
    parser.add_argument("--chainlist-url", default=DEFAULT_CHAINLIST_URL)
    parser.add_argument("--chainlist-path", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    manual = load_json(args.manual)
    chainlist = load_chainlist(args.chainlist_url, args.chainlist_path)
    chains = load_manual_chains(manual, chainlist)
    validate_chains(chains, chainlist)

    outputs = {
        ASSET_CHAINS_PATH: json_dump(asset_chains(chains)),
        GENERATED_MOD_PATH: format_rust(generated_mod()),
        GENERATED_NAMED_PATH: format_rust(generated_named(chains)),
    }

    changed = [path for path, contents in outputs.items() if read_text(path) != contents]
    if args.check:
        if changed:
            for path in changed:
                print(f"{path.relative_to(ROOT)} is not up to date", file=sys.stderr)
            return 1
        return 0

    for path, contents in outputs.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents)
    return 0


def load_chainlist(url: str, path: Path | None) -> dict[int, dict]:
    if path is None:
        request = urllib.request.Request(url, headers={"User-Agent": "alloy-chains-codegen"})
        with urllib.request.urlopen(request) as response:
            data = json.load(response)
    else:
        data = load_json(path)

    if not isinstance(data, list):
        raise ValueError("Chainlist registry must be a JSON array")

    chainlist = {}
    for entry in data:
        chain_id = entry.get("chainId")
        if not isinstance(chain_id, int):
            raise ValueError(f"Chainlist entry has invalid chainId: {entry!r}")
        chainlist[chain_id] = entry
    return chainlist


def load_json(path: Path):
    return json.loads(path.read_text())


def read_text(path: Path) -> str | None:
    try:
        return path.read_text()
    except FileNotFoundError:
        return None


def load_manual_chains(manual: dict, chainlist: dict[int, dict]) -> list[Chain]:
    chains = []
    for raw in manual["chains"]:
        chain_id = raw["chainId"]
        chainlist_entry = chainlist.get(chain_id)
        chains.append(
            Chain(
                chain_id=chain_id,
                internal_id=raw["internalId"],
                name=raw["name"],
                aliases=tuple(raw.get("aliases", [])),
                serde_name=raw.get("serdeName"),
                serde_aliases=tuple(raw.get("serdeAliases", [])),
                average_blocktime_hint=raw.get("averageBlocktimeHint"),
                is_legacy=raw.get("isLegacy", False),
                supports_shanghai=raw.get("supportsShanghai", False),
                is_testnet=raw.get("isTestnet", False),
                native_currency_symbol=raw.get(
                    "nativeCurrencySymbol", chainlist_native_currency_symbol(chainlist_entry)
                ),
                etherscan_api_url=raw.get("etherscanApiUrl"),
                etherscan_base_url=raw.get("etherscanBaseUrl", chainlist_explorer_url(chainlist_entry)),
                etherscan_api_key_name=raw.get("etherscanApiKeyName"),
                tags=frozenset(raw.get("tags", [])),
                wrapped_native_token=raw.get("wrappedNativeToken"),
                manual_only=raw.get("manualOnly", False),
            )
        )
    return chains


def chainlist_native_currency_symbol(entry: dict | None) -> str | None:
    if entry is None:
        return None
    native_currency = entry.get("nativeCurrency")
    if not isinstance(native_currency, dict):
        return None
    symbol = native_currency.get("symbol")
    return symbol if isinstance(symbol, str) else None


def chainlist_explorer_url(entry: dict | None) -> str | None:
    if entry is None:
        return None
    explorers = entry.get("explorers")
    if not isinstance(explorers, list) or not explorers:
        return None
    url = explorers[0].get("url")
    return url.rstrip("/") if isinstance(url, str) else None


def validate_chains(chains: list[Chain], chainlist: dict[int, dict]) -> None:
    seen_ids = set()
    seen_variants = set()
    parse_names = {}
    serde_names = {}

    for chain in chains:
        if chain.chain_id in seen_ids:
            raise ValueError(f"Duplicate chain ID {chain.chain_id}")
        if chain.internal_id in seen_variants:
            raise ValueError(f"Duplicate internal ID {chain.internal_id}")
        seen_ids.add(chain.chain_id)
        seen_variants.add(chain.internal_id)

        if chain.chain_id not in chainlist and not chain.manual_only:
            raise ValueError(
                f"{chain.internal_id} ({chain.chain_id}) is missing from Chainlist; "
                "set manualOnly when this compatibility entry is intentionally local"
            )

        for name in [chain.name, *chain.aliases]:
            previous = parse_names.setdefault(name, chain.internal_id)
            if previous != chain.internal_id:
                raise ValueError(f"Parse name {name!r} is used by {previous} and {chain.internal_id}")

        for name in serde_parse_names(chain):
            previous = serde_names.setdefault(name, chain.internal_id)
            if previous != chain.internal_id:
                raise ValueError(f"Serde name {name!r} is used by {previous} and {chain.internal_id}")


def asset_chains(chains: list[Chain]) -> dict:
    return {
        "chains": {
            str(chain.chain_id): {
                "internalId": chain.internal_id,
                "name": chain.name,
                "averageBlocktimeHint": chain.average_blocktime_hint,
                "isLegacy": chain.is_legacy,
                "supportsShanghai": chain.supports_shanghai,
                "isTestnet": chain.is_testnet,
                "nativeCurrencySymbol": chain.native_currency_symbol,
                "etherscanApiUrl": chain.etherscan_api_url,
                "etherscanBaseUrl": chain.etherscan_base_url,
                "etherscanApiKeyName": chain.etherscan_api_key_name,
            }
            for chain in sorted(chains, key=lambda chain: chain.chain_id)
        }
    }


def generated_mod() -> str:
    return """//! Generated chain registry.

pub(crate) mod named;
"""


def generated_named(chains: list[Chain]) -> str:
    enum_variants = "\n".join(f"    {chain.internal_id} = {chain.chain_id}," for chain in chains)
    variants = "\n".join(f"        Self::{chain.internal_id}," for chain in chains)
    variant_names = "\n".join(f"        {rs_str(chain.name)}," for chain in chains)
    chain_id_arms = "\n".join(
        f"            {chain.chain_id} => Some(Self::{chain.internal_id})," for chain in chains
    )
    as_str_arms = "\n".join(
        f"            Self::{chain.internal_id} => {rs_str(chain.name)}," for chain in chains
    )
    average_blocktime_arms = "\n".join(grouped_option_duration(chains))
    native_currency_symbol_arms = "\n".join(
        grouped_option_str(chains, lambda chain: chain.native_currency_symbol)
    )
    etherscan_url_arms = "\n".join(
        f"            Self::{chain.internal_id} => Some(({rs_str(chain.etherscan_api_url)}, "
        f"{rs_str(chain.etherscan_base_url)})),"
        for chain in chains
        if chain.etherscan_api_url is not None and chain.etherscan_base_url is not None
    )
    etherscan_api_key_name_arms = "\n".join(
        grouped_option_str(chains, lambda chain: chain.etherscan_api_key_name)
    )
    wrapped_native_token_arms = "\n".join(
        f"            Self::{chain.internal_id} => Some(address!({rs_str(chain.wrapped_native_token)})),"
        for chain in chains
        if chain.wrapped_native_token is not None
    )
    serde_arms = "\n".join(
        f"            {match_patterns(serde_parse_names(chain))} => Some(Self::{chain.internal_id}),"
        for chain in chains
    )
    parse_arms = "\n".join(
        f"            {match_patterns(parse_names(chain))} => Ok(Self::{chain.internal_id}),"
        for chain in chains
    )
    parse_aliases = "\n".join(
        f"    (NamedChain::{chain.internal_id}, {rs_str(alias)}),"
        for chain in chains
        for alias in chain.aliases
    )
    serde_aliases = "\n".join(
        f"    (NamedChain::{chain.internal_id}, {rs_str(alias)}),"
        for chain in chains
        for alias in (*chain.serde_aliases, *((chain.serde_name,) if chain.serde_name is not None else ()))
    )

    return f"""// @generated by scripts/update-registry.py.
// Do not edit manually. Update registry/manual.json instead.

use alloy_primitives::{{Address, address}};
use alloc::string::String;
use core::{{cmp::Ordering, fmt, str::FromStr, time::Duration}};

/// An Ethereum EIP-155 chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "snake_case"))]
#[repr(u64)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum NamedChain {{
{enum_variants}
}}

/// Iterator over all named chains.
#[derive(Clone, Debug)]
pub struct NamedChainIter {{
    inner: core::iter::Copied<core::slice::Iter<'static, NamedChain>>,
}}

impl Default for NamedChainIter {{
    #[inline]
    fn default() -> Self {{
        NamedChain::iter()
    }}
}}

impl Iterator for NamedChainIter {{
    type Item = NamedChain;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {{
        self.inner.next()
    }}

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {{
        self.inner.size_hint()
    }}
}}

impl DoubleEndedIterator for NamedChainIter {{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {{
        self.inner.next_back()
    }}
}}

impl ExactSizeIterator for NamedChainIter {{}}
impl core::iter::FusedIterator for NamedChainIter {{}}

impl NamedChain {{
    /// The number of named chains.
    pub const COUNT: usize = {len(chains)};

    /// All named chains in declaration order.
    pub const VARIANTS: &'static [Self] = &[
{variants}
    ];

    /// All named chain string representations in declaration order.
    pub const VARIANT_NAMES: &'static [&'static str] = &[
{variant_names}
    ];

    /// Returns an iterator over all named chains.
    #[inline]
    pub fn iter() -> NamedChainIter {{
        NamedChainIter {{ inner: Self::VARIANTS.iter().copied() }}
    }}

    /// Returns the chain for the given EIP-155 chain ID.
    #[inline]
    pub const fn from_chain_id(id: u64) -> Option<Self> {{
        match id {{
{chain_id_arms}
            _ => None,
        }}
    }}

    /// Returns the string representation of the chain.
    #[inline]
    pub const fn as_str(&self) -> &'static str {{
        match self {{
{as_str_arms}
        }}
    }}

    /// Returns `true` if this chain is Ethereum or an Ethereum testnet.
    #[inline]
    pub const fn is_ethereum(&self) -> bool {{
        {matches_expr('*self', tagged(chains, 'ethereum'))}
    }}

    /// Returns true if the chain contains Optimism configuration.
    #[inline]
    pub const fn is_optimism(self) -> bool {{
        {matches_expr('self', tagged(chains, 'optimism'))}
    }}

    /// Returns true if the chain contains Gnosis configuration.
    #[inline]
    pub const fn is_gnosis(self) -> bool {{
        {matches_expr('self', tagged(chains, 'gnosis'))}
    }}

    /// Returns true if the chain contains Polygon configuration.
    #[inline]
    pub const fn is_polygon(self) -> bool {{
        {matches_expr('self', tagged(chains, 'polygon'))}
    }}

    /// Returns true if the chain contains Arbitrum configuration.
    #[inline]
    pub const fn is_arbitrum(self) -> bool {{
        {matches_expr('self', tagged(chains, 'arbitrum'))}
    }}

    /// Returns true if the chain contains Elastic Network configuration.
    #[inline]
    pub const fn is_elastic(self) -> bool {{
        {matches_expr('self', tagged(chains, 'elastic'))}
    }}

    /// Returns true if the chain contains Tempo configuration.
    #[inline]
    pub const fn is_tempo(self) -> bool {{
        {matches_expr('self', tagged(chains, 'tempo'))}
    }}

    /// Returns true if the chain uses a custom Sourcify-compatible API.
    #[inline]
    pub const fn is_custom_sourcify(self) -> bool {{
        {matches_expr('self', tagged(chains, 'custom-sourcify'))}
    }}

    /// Returns the chain's average blocktime, if applicable.
    #[inline]
    pub const fn average_blocktime_hint(self) -> Option<Duration> {{
        match self {{
{average_blocktime_arms}
            _ => None,
        }}
    }}

    /// Returns whether the chain implements EIP-1559.
    #[inline]
    pub const fn is_legacy(self) -> bool {{
        {matches_expr('self', [c for c in chains if c.is_legacy])}
    }}

    /// Returns whether the chain supports the Shanghai hardfork.
    #[inline]
    pub const fn supports_shanghai(self) -> bool {{
        {matches_expr('self', [c for c in chains if c.supports_shanghai])}
    }}

    /// Returns whether the chain is a testnet.
    #[inline]
    pub const fn is_testnet(self) -> bool {{
        {matches_expr('self', [c for c in chains if c.is_testnet])}
    }}

    /// Returns the symbol of the chain's native currency.
    #[inline]
    pub const fn native_currency_symbol(self) -> Option<&'static str> {{
        match self {{
{native_currency_symbol_arms}
            _ => None,
        }}
    }}

    /// Returns the chain's blockchain explorer and its API URLs.
    #[inline]
    pub const fn etherscan_urls(self) -> Option<(&'static str, &'static str)> {{
        match self {{
{etherscan_url_arms}
            _ => None,
        }}
    }}

    /// Returns the chain's blockchain explorer API key environment variable name.
    #[inline]
    pub const fn etherscan_api_key_name(self) -> Option<&'static str> {{
        match self {{
{etherscan_api_key_name_arms}
            _ => None,
        }}
    }}

    /// Returns the chain's blockchain explorer API key from the environment.
    #[cfg(feature = "std")]
    pub fn etherscan_api_key(self) -> Option<String> {{
        self.etherscan_api_key_name().and_then(|name| std::env::var(name).ok())
    }}

    /// Returns the address of the public DNS node list for the given chain.
    pub fn public_dns_network_protocol(self) -> Option<String> {{
        const DNS_PREFIX: &str =
            "enrtree://AKA3AM6LPBYEUDMVNU3BSVQJ5AD45Y7YPOHJLEF6W26QOE4VTUDPE@";
        if matches!(
            self,
            Self::Mainnet
                | Self::Goerli
                | Self::Sepolia
                | Self::Ropsten
                | Self::Rinkeby
                | Self::Holesky
                | Self::Hoodi
        ) {{
            let mut s = String::with_capacity(DNS_PREFIX.len() + 32);
            s.push_str(DNS_PREFIX);
            s.push_str("all.");
            let chain_str = self.as_ref();
            s.push_str(chain_str);
            let l = s.len();
            s[l - chain_str.len()..].make_ascii_lowercase();
            s.push_str(".ethdisco.net");
            Some(s)
        }} else {{
            None
        }}
    }}

    /// Returns the address of the most popular wrapped native token address.
    #[inline]
    pub const fn wrapped_native_token(self) -> Option<Address> {{
        match self {{
{wrapped_native_token_arms}
            _ => None,
        }}
    }}

    #[cfg(feature = "serde")]
    fn from_serde_str(value: &str) -> Option<Self> {{
        match value {{
{serde_arms}
            _ => None,
        }}
    }}
}}

impl From<NamedChain> for &'static str {{
    #[inline]
    fn from(chain: NamedChain) -> Self {{
        (&chain).into()
    }}
}}

impl From<&NamedChain> for &'static str {{
    #[inline]
    fn from(chain: &NamedChain) -> Self {{
        chain.as_str()
    }}
}}

impl Default for NamedChain {{
    #[inline]
    fn default() -> Self {{
        Self::Mainnet
    }}
}}

macro_rules! impl_into_numeric {{
    ($($t:ty)+) => {{$(
        impl From<NamedChain> for $t {{
            #[inline]
            fn from(chain: NamedChain) -> Self {{
                chain as $t
            }}
        }}
    )+}};
}}

impl_into_numeric!(u64 i64 u128 i128);
#[cfg(target_pointer_width = "64")]
impl_into_numeric!(usize isize);

impl num_enum::TryFromPrimitive for NamedChain {{
    type Primitive = u64;
    type Error = num_enum::TryFromPrimitiveError<Self>;

    const NAME: &'static str = "NamedChain";

    #[inline]
    fn try_from_primitive(number: Self::Primitive) -> Result<Self, Self::Error> {{
        Self::from_chain_id(number).ok_or_else(|| num_enum::TryFromPrimitiveError::new(number))
    }}
}}

impl TryFrom<u64> for NamedChain {{
    type Error = num_enum::TryFromPrimitiveError<Self>;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {{
        num_enum::TryFromPrimitive::try_from_primitive(value)
    }}
}}

macro_rules! impl_try_from_numeric {{
    ($($native:ty)+) => {{
        $(
            impl TryFrom<$native> for NamedChain {{
                type Error = num_enum::TryFromPrimitiveError<NamedChain>;

                #[inline]
                fn try_from(value: $native) -> Result<Self, Self::Error> {{
                    (value as u64).try_into()
                }}
            }}
        )+
    }};
}}

impl_try_from_numeric!(u8 i8 u16 i16 u32 i32 usize isize);

impl fmt::Display for NamedChain {{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        self.as_str().fmt(f)
    }}
}}

impl AsRef<str> for NamedChain {{
    #[inline]
    fn as_ref(&self) -> &str {{
        self.as_str()
    }}
}}

impl FromStr for NamedChain {{
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {{
        match s {{
{parse_arms}
            _ => Err(strum::ParseError::VariantNotFound),
        }}
    }}
}}

impl TryFrom<&str> for NamedChain {{
    type Error = strum::ParseError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {{
        value.parse()
    }}
}}

impl PartialEq<u64> for NamedChain {{
    #[inline]
    fn eq(&self, other: &u64) -> bool {{
        (*self as u64) == *other
    }}
}}

impl PartialOrd<u64> for NamedChain {{
    #[inline]
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {{
        (*self as u64).partial_cmp(other)
    }}
}}

impl strum::EnumCount for NamedChain {{
    const COUNT: usize = Self::COUNT;
}}

impl strum::VariantArray for NamedChain {{
    const VARIANTS: &'static [Self] = Self::VARIANTS;
}}

impl strum::VariantNames for NamedChain {{
    const VARIANTS: &'static [&'static str] = Self::VARIANT_NAMES;
}}

impl strum::IntoEnumIterator for NamedChain {{
    type Iterator = NamedChainIter;

    #[inline]
    fn iter() -> Self::Iterator {{
        Self::iter()
    }}
}}

#[cfg(feature = "serde")]
impl serde::Serialize for NamedChain {{
    #[inline]
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {{
        s.serialize_str(self.as_ref())
    }}
}}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NamedChain {{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {{
        struct NamedChainVisitor;

        impl serde::de::Visitor<'_> for NamedChainVisitor {{
            type Value = NamedChain;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {{
                formatter.write_str("a named chain")
            }}

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {{
                NamedChain::from_serde_str(value)
                    .ok_or_else(|| serde::de::Error::unknown_variant(value, NamedChain::VARIANT_NAMES))
            }}
        }}

        deserializer.deserialize_str(NamedChainVisitor)
    }}
}}

#[cfg(feature = "rlp")]
impl alloy_rlp::Encodable for NamedChain {{
    #[inline]
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {{
        (*self as u64).encode(out)
    }}

    #[inline]
    fn length(&self) -> usize {{
        (*self as u64).length()
    }}
}}

#[cfg(feature = "rlp")]
impl alloy_rlp::Decodable for NamedChain {{
    #[inline]
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {{
        let n = u64::decode(buf)?;
        Self::try_from(n).map_err(|_| alloy_rlp::Error::Overflow)
    }}
}}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for NamedChain {{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {{
        let idx = u.choose_index(Self::COUNT)?;
        Ok(Self::VARIANTS[idx])
    }}
}}

#[cfg(test)]
pub(crate) const PARSE_ALIASES: &[(NamedChain, &str)] = &[
{parse_aliases}
];

#[cfg(all(test, feature = "serde"))]
pub(crate) const SERDE_ALIASES: &[(NamedChain, &str)] = &[
{serde_aliases}
];
"""


def tagged(chains: list[Chain], tag: str) -> list[Chain]:
    return [chain for chain in chains if tag in chain.tags]


def grouped_option_duration(chains: list[Chain]) -> list[str]:
    groups: dict[int, list[Chain]] = defaultdict(list)
    for chain in chains:
        if chain.average_blocktime_hint is not None:
            groups[chain.average_blocktime_hint].append(chain)
    return [
        f"            {variant_pattern(group)} => Some(Duration::from_millis({value})),"
        for value, group in sorted(groups.items())
    ]


def grouped_option_str(chains: list[Chain], get_value) -> list[str]:
    groups: dict[str, list[Chain]] = defaultdict(list)
    for chain in chains:
        value = get_value(chain)
        if value is not None:
            groups[value].append(chain)
    return [
        f"            {variant_pattern(group)} => Some({rs_str(value)}),"
        for value, group in sorted(groups.items())
    ]


def parse_names(chain: Chain) -> list[str]:
    return unique([chain.name, *chain.aliases])


def serde_parse_names(chain: Chain) -> list[str]:
    serde_name = chain.serde_name or snake_case(chain.internal_id)
    return unique([*parse_names(chain), serde_name, *chain.serde_aliases])


def snake_case(value: str) -> str:
    output = []
    chars = list(value)
    for index, char in enumerate(chars):
        if char.isupper() and index > 0:
            prev = chars[index - 1]
            next_char = chars[index + 1] if index + 1 < len(chars) else ""
            if prev.islower() or prev.isdigit() or (prev.isupper() and next_char.islower()):
                output.append("_")
        output.append(char.lower())
    return "".join(output)


def unique(items: list[str]) -> list[str]:
    seen = set()
    output = []
    for item in items:
        if item not in seen:
            seen.add(item)
            output.append(item)
    return output


def matches_expr(expr: str, chains: list[Chain]) -> str:
    if not chains:
        return "false"
    return f"matches!({expr}, {variant_pattern(chains)})"


def variant_pattern(chains: list[Chain]) -> str:
    return " | ".join(f"Self::{chain.internal_id}" for chain in chains)


def match_patterns(values: list[str]) -> str:
    return " | ".join(rs_str(value) for value in values)


def rs_str(value: str) -> str:
    return json.dumps(value)


def json_dump(value) -> str:
    return json.dumps(value, indent=2) + "\n"


def format_rust(source: str) -> str:
    output = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=source,
        text=True,
        capture_output=True,
        check=True,
    )
    return output.stdout


if __name__ == "__main__":
    raise SystemExit(main())
