//! Runtime chain registry.

extern crate alloc;
use crate::{Chain, MAINNET, SEPOLIA};
use alloc::collections::BTreeMap;

/// Runtime chain registry.
#[derive(Default)]
pub struct ChainRegistry {
    /// The registry of chains.
    pub chains: BTreeMap<u64, Chain>,
}

impl ChainRegistry {
    /// Instanciates a new ChainRegistry.
    pub fn new() -> Self {
        let mut chains: BTreeMap<u64, Chain> = BTreeMap::new();
        chains.insert(MAINNET.id, MAINNET);
        chains.insert(SEPOLIA.id, SEPOLIA);
        ChainRegistry { chains }
    }

    /// Returns a reference to the chain with the given ID.
    pub fn get(&self, id: u64) -> Option<&Chain> {
        self.chains.get(&id)
    }

    /// Returns the Ethereum Mainnet chain.
    pub fn mainnet() -> Chain {
        MAINNET
    }

    /// Returns the Sepolia mainnet chain.
    pub fn sepolia() -> Chain {
        SEPOLIA
    }

    /// Adds a chain to the registry.
    pub fn add_chain(&mut self, chain: Chain) {
        self.chains.insert(chain.id, chain);
    }

    /// Removes a chain from the registry.
    pub fn remove_chain(&mut self, id: u64) -> Option<Chain> {
        self.chains.remove(&id)
    }
}
