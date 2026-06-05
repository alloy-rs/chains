#!/usr/bin/env python3
"""Update generated chain registry artifacts."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import urllib.request
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CHAINLIST_URL = "https://chainid.network/chains.json"
MANUAL_PATH = ROOT / "registry" / "manual.json"
ASSET_CHAINS_PATH = ROOT / "assets" / "chains.json"
GENERATED_MOD_PATH = ROOT / "src" / "generated" / "mod.rs"
GENERATED_NAMED_PATH = ROOT / "src" / "generated" / "named.rs"
PHF_CODEGEN_PATH = ROOT / "scripts" / "phf-codegen.rs"
STATIC_STR_NONE = "N"
NO_WRAPPED_NATIVE_TOKEN = "NO_WRAPPED_NATIVE_TOKEN"
MATCHES_TAG_LIMIT = 5
BASE_CHAIN_FLAGS = (
    ("legacy", "FLAG_LEGACY"),
    ("supports_shanghai", "FLAG_SUPPORTS_SHANGHAI"),
    ("testnet", "FLAG_TESTNET"),
)
CHAIN_TAG_FLAGS = (
    ("ethereum", "FLAG_ETHEREUM"),
    ("optimism", "FLAG_OPTIMISM"),
    ("gnosis", "FLAG_GNOSIS"),
    ("polygon", "FLAG_POLYGON"),
    ("arbitrum", "FLAG_ARBITRUM"),
    ("elastic", "FLAG_ELASTIC"),
    ("tempo", "FLAG_TEMPO"),
    ("custom-sourcify", "FLAG_CUSTOM_SOURCIFY"),
)


@dataclass(frozen=True)
class StaticStr:
    offset: int
    length: int


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


class StaticStringTable:
    def __init__(self):
        self.data = bytearray()
        self.offsets: dict[bytes, int] = {}

    def add(self, value: str | None) -> StaticStr | None:
        if value is None:
            return None

        encoded = value.encode()
        if len(encoded) > 255:
            raise ValueError(f"Static string is too long for compact storage: {value!r}")

        offset = self.offsets.get(encoded)
        if offset is None:
            offset = self.data.find(encoded)

        if offset < 0:
            offset = len(self.data)
            self.data.extend(encoded)
            self.offsets[encoded] = offset

        if offset > 0xFFFF_FFFF:
            raise ValueError("Static string table is too large for compact storage")
        return StaticStr(offset, len(encoded))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manual", type=Path, default=MANUAL_PATH)
    parser.add_argument("--chainlist-url", default=DEFAULT_CHAINLIST_URL)
    parser.add_argument("--chainlist-path", type=Path)
    parser.add_argument(
        "--no-fetch",
        action="store_true",
        help="Use checked-in assets/chains.json defaults instead of fetching Chainlist.",
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    manual = load_json(args.manual)
    chainlist = load_chainlist(args.chainlist_url, args.chainlist_path, args.no_fetch)
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


def load_chainlist(url: str, path: Path | None, no_fetch: bool) -> dict[int, dict]:
    if path is not None:
        data = load_json(path)
    elif no_fetch:
        return load_asset_chainlist()
    else:
        request = urllib.request.Request(url, headers={"User-Agent": "alloy-chains-codegen"})
        with urllib.request.urlopen(request) as response:
            data = json.load(response)

    if not isinstance(data, list):
        raise ValueError("Chainlist registry must be a JSON array")

    chainlist = {}
    for entry in data:
        chain_id = entry.get("chainId")
        if not isinstance(chain_id, int):
            raise ValueError(f"Chainlist entry has invalid chainId: {entry!r}")
        chainlist[chain_id] = entry
    return chainlist


def load_asset_chainlist() -> dict[int, dict]:
    data = load_json(ASSET_CHAINS_PATH)
    chains = data.get("chains") if isinstance(data, dict) else None
    if not isinstance(chains, dict):
        raise ValueError("Asset chain registry must contain a JSON object at `chains`")

    chainlist = {}
    for raw_chain_id, chain in chains.items():
        if not isinstance(chain, dict):
            raise ValueError(f"Asset chain entry has invalid data: {chain!r}")

        try:
            chain_id = int(raw_chain_id)
        except ValueError as error:
            raise ValueError(f"Asset chain entry has invalid chain ID: {raw_chain_id!r}") from error

        entry = {"chainId": chain_id}

        symbol = chain.get("nativeCurrencySymbol")
        if symbol is not None:
            if not isinstance(symbol, str):
                raise ValueError(f"Asset chain {raw_chain_id} has invalid native currency symbol")
            entry["nativeCurrency"] = {"symbol": symbol}

        explorer_url = chain.get("etherscanBaseUrl")
        if explorer_url is not None:
            if not isinstance(explorer_url, str):
                raise ValueError(f"Asset chain {raw_chain_id} has invalid explorer URL")
            entry["explorers"] = [{"url": explorer_url}]

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
    if len(chains) > 0xFFFF:
        raise ValueError("Too many chains for compact index storage")
    if len(chains) > 0xFF:
        chain_index_type = "u16"
        chain_index_from = "index as usize"
    else:
        chain_index_type = "u8"
        chain_index_from = "index as usize"

    string_table = StaticStringTable()
    chain_indexes = {chain.internal_id: index for index, chain in enumerate(chains)}
    wrapped_native_tokens = unique(
        [chain.wrapped_native_token for chain in chains if chain.wrapped_native_token is not None]
    )
    if len(wrapped_native_tokens) > 0xFF:
        raise ValueError("Too many wrapped native tokens for compact storage")
    wrapped_native_token_indexes = {token: index for index, token in enumerate(wrapped_native_tokens)}

    enum_variants = "\n".join(f"    {chain.internal_id} = {chain.chain_id}," for chain in chains)
    variants = "\n".join(f"        Self::{chain.internal_id}," for chain in chains)
    chain_id_arms = "\n".join(
        f"            {chain.chain_id} => Some(Self::{chain.internal_id})," for chain in chains
    )
    chain_data_arms = "\n".join(
        f"            Self::{chain.internal_id} => CHAIN_DATA[{index}],"
        for index, chain in enumerate(chains)
    )
    stored_tag_flags = stored_chain_tag_flags(chains)
    flag_type, flag_consts = generated_flag_consts(stored_tag_flags)
    chain_data = "\n".join(
        "    d(["
        f"{static_str(string_table.add(chain.name))}, "
        f"{static_str(string_table.add(chain.native_currency_symbol))}, "
        f"{static_str(string_table.add(chain.etherscan_api_url))}, "
        f"{static_str(string_table.add(chain.etherscan_base_url))}, "
        f"{static_str(string_table.add(chain.etherscan_api_key_name))}"
        "], "
        f"{average_blocktime_millis(chain)}, "
        f"{chain_flags(chain, stored_tag_flags)}, "
        f"{wrapped_native_token_index(chain, wrapped_native_token_indexes)}"
        "),"
        for chain in chains
    )
    string_data = rs_byte_str(string_table.data)
    wrapped_native_token_data = "\n".join(
        f"    address!({rs_str(token)})," for token in wrapped_native_tokens
    )
    parse_aliases = "\n".join(
        f"    (NamedChain::{chain.internal_id}, {rs_str(alias)}),"
        for chain in chains
        for alias in chain.aliases
    )
    serde_aliases = "\n".join(
        f"    (NamedChain::{chain.internal_id}, {rs_str(alias)}),"
        for chain in chains
        for alias in serde_extra_names(chain)
    )
    parse_entries = [
        (name, chain_indexes[chain.internal_id])
        for chain in chains
        for name in parse_names(chain)
    ]
    serde_entries = [
        (name, chain_indexes[chain.internal_id])
        for chain in chains
        for name in serde_extra_names(chain)
    ]
    chain_id_entries = [
        (str(chain.chain_id), chain_indexes[chain.internal_id])
        for chain in chains
    ]
    phf_maps = generate_phf_maps(chain_id_entries, parse_entries, serde_entries)
    chain_data_len = len(chains)
    wrapped_native_token_len = len(wrapped_native_tokens)
    string_data_len = len(string_table.data)
    if string_data_len > 0xFFFF_FFFF:
        raise ValueError("Static string data is too large for compact storage")
    tag_predicates = {
        tag.replace("-", "_"): tag_predicate_expr(chains, stored_tag_flags, tag, "self")
        for tag, _flag in CHAIN_TAG_FLAGS
    }

    return f"""// @generated by scripts/update-registry.py.
// Do not edit manually. Update registry/manual.json instead.

use alloy_primitives::{{Address, address}};
use alloc::string::String;
use core::{{cmp::Ordering, fmt, str::FromStr, time::Duration}};

type ChainIndex = {chain_index_type};

{flag_consts}
const {NO_WRAPPED_NATIVE_TOKEN}: u8 = u8::MAX;
const {STATIC_STR_NONE}: StaticStr = StaticStr::NONE;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct StaticStr {{
    offset: u32,
    len: u8,
}}

impl StaticStr {{
    const NONE: Self = Self {{ offset: u32::MAX, len: 0 }};

    #[inline]
    const fn get(self) -> Option<&'static str> {{
        if self.offset == u32::MAX {{
            None
        }} else {{
            Some(self.get_unchecked())
        }}
    }}

    #[inline]
    const fn get_unchecked(self) -> &'static str {{
        let ptr = unsafe {{ STRING_DATA.as_ptr().add(self.offset as usize) }};
        let bytes = unsafe {{ core::slice::from_raw_parts(ptr, self.len as usize) }};
        unsafe {{ core::str::from_utf8_unchecked(bytes) }}
    }}
}}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct ChainData {{
    name: StaticStr,
    average_blocktime_millis: u16,
    flags: {flag_type},
    native_currency_symbol: StaticStr,
    etherscan_api_url: StaticStr,
    etherscan_base_url: StaticStr,
    etherscan_api_key_name: StaticStr,
    wrapped_native_token: u8,
}}

const fn s(offset: u32, len: u8) -> StaticStr {{
    StaticStr {{ offset, len }}
}}

const fn d(
    strings: [StaticStr; 5],
    average_blocktime_millis: u16,
    flags: {flag_type},
    wrapped_native_token: u8,
) -> ChainData {{
    let [name, native_currency_symbol, etherscan_api_url, etherscan_base_url, etherscan_api_key_name] =
        strings;
    ChainData {{
        name,
        average_blocktime_millis,
        flags,
        native_currency_symbol,
        etherscan_api_url,
        etherscan_base_url,
        etherscan_api_key_name,
        wrapped_native_token,
    }}
}}

const fn variant_names() -> [&'static str; {chain_data_len}] {{
    let mut names = [""; {chain_data_len}];
    let mut index = 0;
    while index < CHAIN_DATA.len() {{
        names[index] = CHAIN_DATA[index].name.get_unchecked();
        index += 1;
    }}
    names
}}

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

/// Error returned when parsing a named chain from a string fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParseNamedChainError;

impl fmt::Display for ParseNamedChainError {{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        f.write_str("matching variant not found")
    }}
}}

#[cfg(feature = "std")]
impl std::error::Error for ParseNamedChainError {{}}

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
    pub const COUNT: usize = {chain_data_len};

    /// All named chains in declaration order.
    pub const VARIANTS: &'static [Self] = &[
{variants}
    ];

    /// All named chain string representations in declaration order.
    pub const VARIANT_NAMES: &'static [&'static str] = &VARIANT_NAMES_DATA;

    /// Returns an iterator over all named chains.
    #[inline]
    pub fn iter() -> NamedChainIter {{
        NamedChainIter {{ inner: Self::VARIANTS.iter().copied() }}
    }}

    #[inline]
    const fn from_index(index: ChainIndex) -> Self {{
        Self::VARIANTS[{chain_index_from}]
    }}

    #[inline]
    const fn data(self) -> ChainData {{
        match self {{
{chain_data_arms}
        }}
    }}

    #[inline]
    fn from_chain_id_map(id: u64) -> Option<Self> {{
        CHAIN_IDS.get(&id).copied().map(Self::from_index)
    }}

    #[inline]
    const fn has_flag(self, flag: {flag_type}) -> bool {{
        self.data().flags & flag != 0
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
        self.data().name.get_unchecked()
    }}

    /// Returns `true` if this chain is Ethereum or an Ethereum testnet.
    #[inline]
    pub const fn is_ethereum(&self) -> bool {{
        {tag_predicates["ethereum"]}
    }}

    /// Returns true if the chain contains Optimism configuration.
    #[inline]
    pub const fn is_optimism(self) -> bool {{
        {tag_predicates["optimism"]}
    }}

    /// Returns true if the chain contains Gnosis configuration.
    #[inline]
    pub const fn is_gnosis(self) -> bool {{
        {tag_predicates["gnosis"]}
    }}

    /// Returns true if the chain contains Polygon configuration.
    #[inline]
    pub const fn is_polygon(self) -> bool {{
        {tag_predicates["polygon"]}
    }}

    /// Returns true if the chain contains Arbitrum configuration.
    #[inline]
    pub const fn is_arbitrum(self) -> bool {{
        {tag_predicates["arbitrum"]}
    }}

    /// Returns true if the chain contains Elastic Network configuration.
    #[inline]
    pub const fn is_elastic(self) -> bool {{
        {tag_predicates["elastic"]}
    }}

    /// Returns true if the chain contains Tempo configuration.
    #[inline]
    pub const fn is_tempo(self) -> bool {{
        {tag_predicates["tempo"]}
    }}

    /// Returns true if the chain uses a custom Sourcify-compatible API.
    #[inline]
    pub const fn is_custom_sourcify(self) -> bool {{
        {tag_predicates["custom_sourcify"]}
    }}

    /// Returns the chain's average blocktime, if applicable.
    #[inline]
    pub const fn average_blocktime_hint(self) -> Option<Duration> {{
        let millis = self.data().average_blocktime_millis;
        if millis == 0 {{
            None
        }} else {{
            Some(Duration::from_millis(millis as u64))
        }}
    }}

    /// Returns whether the chain implements EIP-1559.
    #[inline]
    pub const fn is_legacy(self) -> bool {{
        self.has_flag(FLAG_LEGACY)
    }}

    /// Returns whether the chain supports the Shanghai hardfork.
    #[inline]
    pub const fn supports_shanghai(self) -> bool {{
        self.has_flag(FLAG_SUPPORTS_SHANGHAI)
    }}

    /// Returns whether the chain is a testnet.
    #[inline]
    pub const fn is_testnet(self) -> bool {{
        self.has_flag(FLAG_TESTNET)
    }}

    /// Returns the symbol of the chain's native currency.
    #[inline]
    pub const fn native_currency_symbol(self) -> Option<&'static str> {{
        self.data().native_currency_symbol.get()
    }}

    /// Returns the chain's blockchain explorer and its API URLs.
    #[inline]
    pub const fn etherscan_urls(self) -> Option<(&'static str, &'static str)> {{
        let data = self.data();
        match (data.etherscan_api_url.get(), data.etherscan_base_url.get()) {{
            (Some(api), Some(base)) => Some((api, base)),
            _ => None,
        }}
    }}

    /// Returns the chain's blockchain explorer API key environment variable name.
    #[inline]
    pub const fn etherscan_api_key_name(self) -> Option<&'static str> {{
        self.data().etherscan_api_key_name.get()
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
        let index = self.data().wrapped_native_token;
        if index == {NO_WRAPPED_NATIVE_TOKEN} {{
            None
        }} else {{
            Some(WRAPPED_NATIVE_TOKENS[index as usize])
        }}
    }}

    #[cfg(feature = "serde")]
    fn from_serde_str(value: &str) -> Option<Self> {{
        PARSE_NAMES
            .get(value)
            .or_else(|| SERDE_NAMES.get(value))
            .copied()
            .map(Self::from_index)
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
        Self::from_chain_id_map(number).ok_or_else(|| num_enum::TryFromPrimitiveError::new(number))
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
    type Err = ParseNamedChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {{
        PARSE_NAMES
            .get(s)
            .copied()
            .map(Self::from_index)
            .ok_or(ParseNamedChainError)
    }}
}}

impl TryFrom<&str> for NamedChain {{
    type Error = ParseNamedChainError;

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

static STRING_DATA: &[u8] = {string_data};

static CHAIN_DATA: [ChainData; {chain_data_len}] = [
{chain_data}
];

static WRAPPED_NATIVE_TOKENS: [Address; {wrapped_native_token_len}] = [
{wrapped_native_token_data}
];

static VARIANT_NAMES_DATA: [&str; {chain_data_len}] = variant_names();

{phf_maps}
"""


def average_blocktime_millis(chain: Chain) -> int:
    value = chain.average_blocktime_hint or 0
    if value > 0xFFFF:
        raise ValueError(f"{chain.internal_id} average blocktime is too large for compact storage")
    return value


def stored_chain_tag_flags(chains: list[Chain]) -> list[tuple[str, str]]:
    return [
        (tag, flag)
        for tag, flag in CHAIN_TAG_FLAGS
        if len(chains_with_tag(chains, tag)) > MATCHES_TAG_LIMIT
    ]


def generated_flag_consts(stored_tag_flags: list[tuple[str, str]]) -> tuple[str, str]:
    flags = [flag for _name, flag in [*BASE_CHAIN_FLAGS, *stored_tag_flags]]
    if len(flags) <= 8:
        flag_type = "u8"
    elif len(flags) <= 16:
        flag_type = "u16"
    elif len(flags) <= 32:
        flag_type = "u32"
    else:
        raise ValueError("Too many chain flags for compact storage")

    return flag_type, "\n".join(
        f"const {flag}: {flag_type} = 1 << {index};" for index, flag in enumerate(flags)
    )


def tag_predicate_expr(
    chains: list[Chain],
    stored_tag_flags: list[tuple[str, str]],
    tag: str,
    receiver: str,
) -> str:
    stored_flags = dict(stored_tag_flags)
    if flag := stored_flags.get(tag):
        return f"{receiver}.has_flag({flag})"

    return matches_expr(chains_with_tag(chains, tag), receiver)


def matches_expr(chains: list[Chain], receiver: str) -> str:
    if not chains:
        return "false"
    variants = " | ".join(f"Self::{chain.internal_id}" for chain in chains)
    return f"matches!({receiver}, {variants})"


def chains_with_tag(chains: list[Chain], tag: str) -> list[Chain]:
    return [chain for chain in chains if tag in chain.tags]


def chain_flags(chain: Chain, stored_tag_flags: list[tuple[str, str]]) -> str:
    flags = []
    if chain.is_legacy:
        flags.append("FLAG_LEGACY")
    if chain.supports_shanghai:
        flags.append("FLAG_SUPPORTS_SHANGHAI")
    if chain.is_testnet:
        flags.append("FLAG_TESTNET")
    for tag, flag in stored_tag_flags:
        if tag in chain.tags:
            flags.append(flag)
    return " | ".join(flags) if flags else "0"


def static_str(value: StaticStr | None) -> str:
    if value is None:
        return STATIC_STR_NONE
    return f"s({value.offset}, {value.length})"


def wrapped_native_token_index(chain: Chain, indexes: dict[str, int]) -> str:
    if chain.wrapped_native_token is None:
        return NO_WRAPPED_NATIVE_TOKEN
    return str(indexes[chain.wrapped_native_token])


def generate_phf_maps(
    chain_id_entries: list[tuple[str, int]],
    parse_entries: list[tuple[str, int]],
    serde_entries: list[tuple[str, int]],
) -> str:
    lines = ["map_u64\tCHAIN_IDS"]
    lines.extend(phf_entry_lines(chain_id_entries))
    lines.append("end")
    lines.append("map\tPARSE_NAMES\t")
    lines.extend(phf_entry_lines(parse_entries))
    lines.append("end")
    lines.append('map\tSERDE_NAMES\t#[cfg(feature = "serde")]')
    lines.extend(phf_entry_lines(serde_entries))
    lines.append("end")

    output = subprocess.run(
        ["cargo", "-Zscript", str(PHF_CODEGEN_PATH)],
        input="\n".join(lines) + "\n",
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    return output.stdout.strip()


def phf_entry_lines(entries: list[tuple[str, int]]) -> list[str]:
    lines = []
    for key, value in entries:
        if "\t" in key or "\n" in key:
            raise ValueError(f"PHF key contains unsupported whitespace: {key!r}")
        lines.append(f"{key}\t{value}")
    return lines


def serde_extra_names(chain: Chain) -> list[str]:
    parse = set(parse_names(chain))
    serde_name = chain.serde_name or snake_case(chain.internal_id)
    return [name for name in unique([serde_name, *chain.serde_aliases]) if name not in parse]


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


def rs_str(value: str) -> str:
    return json.dumps(value)


def rs_byte_str(value: bytes | bytearray) -> str:
    output = ['b"']
    for byte in value:
        if byte == 0x09:
            output.append(r"\t")
        elif byte == 0x0A:
            output.append(r"\n")
        elif byte == 0x0D:
            output.append(r"\r")
        elif byte == 0x22:
            output.append(r'\"')
        elif byte == 0x5C:
            output.append(r"\\")
        elif 0x20 <= byte <= 0x7E:
            output.append(chr(byte))
        else:
            output.append(f"\\x{byte:02x}")
    output.append('"')
    return "".join(output)


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
