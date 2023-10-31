//! Canonical representations of Ethereum Mainnet and Ethereum Sepolia.

use crate::Chain;

/// Canonical Ethereum Mainnet Chain representation.
/// Contains the correct chain ID and name.
pub const MAINNET: Chain = Chain {
    id: 1,
    name: "mainnet",
};

/// Canonical Ethereum Sepolia Chain representation.
/// Contains the correct chain ID and name.
pub const SEPOLIA: Chain = Chain {
    /// Instanciates a new Sepolia Chain.
    id: 11155111,
    name: "sepolia",
};