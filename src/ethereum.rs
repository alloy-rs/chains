//! Canonical representations of Ethereum Mainnet and Ethereum Sepolia.

extern crate alloc;

use crate::{Chain, ChainMetadata};
use alloc::string::{String, ToString};

/// Canonical Ethereum Mainnet Chain representation.
/// Contains the correct chain ID and name,
/// along with information about EIP 1559 and hard-fork support.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mainnet {
    id: u64,
    name: String,
}

impl Mainnet {
    /// Instanciates a new Mainnet Chain.
    pub fn new() -> Self {
        Mainnet {
            id: 1,
            name: "Mainnet".to_string(),
        }
    }
}

impl Chain for Mainnet {
    fn chain_id(&self) -> u64 {
        self.id
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ChainMetadata for Mainnet {
    fn is_legacy(&self) -> bool {
        false
    }

    fn supports_push0(&self) -> bool {
        true
    }
}

/// Canonical Ethereum Sepolia Chain representation.
/// Contains the correct chain ID and name,
/// along with information about EIP 1559 and hard-fork support.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sepolia {
    id: u64,
    name: String,
}

impl Sepolia {
    /// Instanciates a new Sepolia Chain.
    pub fn new() -> Self {
        Sepolia {
            id: 11155111,
            name: "Sepolia".to_string(),
        }
    }
}

impl Chain for Sepolia {
    fn chain_id(&self) -> u64 {
        self.id
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ChainMetadata for Sepolia {
    fn is_legacy(&self) -> bool {
        false
    }

    fn supports_push0(&self) -> bool {
        true
    }
}
