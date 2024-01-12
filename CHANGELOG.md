# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/alloy-rs/chains/compare/v0.1.9...HEAD)

## [0.1.9](https://github.com/foundry-rs/compilers/releases/tag/v0.1.9) - 2024-01-12

### Miscellaneous Tasks

- Release 0.1.9
- Also add base to supports shanghai ([#29](https://github.com/foundry-rs/compilers/issues/29))

## [0.1.8](https://github.com/foundry-rs/compilers/releases/tag/v0.1.8) - 2024-01-12

### Features

- Add public goods network and pgn sepolia to chainsˆ ([#25](https://github.com/foundry-rs/compilers/issues/25))
- Add viction chain ([#26](https://github.com/foundry-rs/compilers/issues/26))

### Miscellaneous Tasks

- Release 0.1.8
- Add holesky to dns match ([#28](https://github.com/foundry-rs/compilers/issues/28))
- Mark Optimism + Holesky as supports shanghai ([#27](https://github.com/foundry-rs/compilers/issues/27))

## [0.1.7](https://github.com/foundry-rs/compilers/releases/tag/v0.1.7) - 2023-12-29

### Features

- Add base sepolia ([#24](https://github.com/foundry-rs/compilers/issues/24))

### Miscellaneous Tasks

- Release 0.1.7

## [0.1.6](https://github.com/foundry-rs/compilers/releases/tag/v0.1.6) - 2023-12-23

### Features

- Add PartialEq<u64> impls ([#22](https://github.com/foundry-rs/compilers/issues/22))

### Miscellaneous Tasks

- Release 0.1.6
- Add arbitrum one alias ([#20](https://github.com/foundry-rs/compilers/issues/20))

### Styling

- Unify etherscan url format ([#23](https://github.com/foundry-rs/compilers/issues/23))

## [0.1.5](https://github.com/foundry-rs/compilers/releases/tag/v0.1.5) - 2023-12-12

### Added

- `is_testnet` and `native_currency_symbol` methods ([#14])
- Chain specification ([#19])

[#14]: https://github.com/alloy-rs/chains/pull/14
[#19]: https://github.com/alloy-rs/chains/pull/19

## [0.1.4](https://github.com/alloy-rs/chains/releases/tag/v0.1.4) - 2023-12-01

### Added

- Scroll sepolia chain ([#17])

### Changed

- OP stack blocktimes ([#12])
- Shanghai support for Polygon ([#16])

[#12]: https://github.com/alloy-rs/chains/pull/12
[#16]: https://github.com/alloy-rs/chains/pull/16
[#17]: https://github.com/alloy-rs/chains/pull/17

## [0.1.3](https://github.com/alloy-rs/chains/releases/tag/v0.1.3) - 2023-11-20

### Added

- Zora chain ([#8])
- `supports_shanghai` ([#9])

### Changed

- Metis to return `true` for `is_legacy` ([#11])

### Deprecated

- `supports_push0` ([#9])

[#8]: https://github.com/alloy-rs/chains/pull/8
[#9]: https://github.com/alloy-rs/chains/pull/9
[#11]: https://github.com/alloy-rs/chains/pull/11

## [0.1.2](https://github.com/alloy-rs/chains/releases/tag/v0.1.2) - 2023-11-15

### Fixed

- Serde implementation for `Chain` ([#7])

[#7]: https://github.com/alloy-rs/chains/pull/7

## [0.1.1](https://github.com/alloy-rs/chains/releases/tag/v0.1.1) - 2023-11-14

### Added

- More implementations and delegated methods to `Chain` ([#6])

[#6]: https://github.com/alloy-rs/chains/pull/6

## [0.1.0](https://github.com/alloy-rs/chains/releases/tag/v0.1.0) - 2023-11-14

### Added

- Initial release, forked from [`ethers_core::types::Chain`](https://github.com/gakonst/ethers-rs/blob/f97bb1db0e34727d7d74549bba5f6e246d760c13/ethers-core/src/types/chain.rs#L55) ([#2]) as `NamedChain` ([#3]), and [`reth_primitives::Chain`](https://github.com/paradigmxyz/reth/blob/8ecd90b884d1ae9cf9119a743b658a4a6dd110c1/crates/primitives/src/chain/mod.rs#L97) ([#4])

### Changed

- split Chain into a struct and ChainKind enum ([#5])

[#2]: https://github.com/alloy-rs/chains/pull/2
[#3]: https://github.com/alloy-rs/chains/pull/3
[#4]: https://github.com/alloy-rs/chains/pull/4
[#5]: https://github.com/alloy-rs/chains/pull/5
