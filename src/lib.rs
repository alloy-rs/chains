//! # Chains.
//!
//! Base Chain struct and canonical representations of Ethereum Mainnet and Testnet chains.

#![warn(
    missing_docs,
    unreachable_pub,
    unused_crate_dependencies,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Main Chain trait.
pub mod chain;
pub use chain::*;

/// Canonical representations of Ethereum-related chains.
mod ethereum;
pub use ethereum::{MAINNET, SEPOLIA};

/// Runtime chain registry.
mod registry;
pub use registry::ChainRegistry;