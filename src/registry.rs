//! Runtime chain registry.
//!
//!

extern crate alloc;
use crate::{Chain, Mainnet, Sepolia};
use alloc::{boxed::Box, collections::BTreeMap};

/// Runtime chain registry.
pub struct ChainRegistry {
    /// The registry of chains.
    pub chains: BTreeMap<u64, Box<dyn Chain>>,
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainRegistry {
    /// Instanciates a new ChainRegistry.
    pub fn new() -> Self {
        let mut chains: BTreeMap<u64, Box<dyn Chain>> = BTreeMap::new();
        let mainnet = Mainnet::new();
        let sepolia = Sepolia::new();
        chains.insert(mainnet.chain_id(), Box::new(mainnet));
        chains.insert(sepolia.chain_id(), Box::new(sepolia));
        ChainRegistry { chains }
    }

    /// Returns a reference to the chain with the given ID.
    pub fn get(&self, id: u64) -> Option<&Box<dyn Chain>> {
        self.chains.get(&id)
    }

    /// Returns the Ethereum Mainnet chain.
    pub fn mainnet() -> Mainnet {
        Mainnet::new()
    }

    /// Returns the Sepolia mainnet chain.
    pub fn sepolia() -> Sepolia {
        Sepolia::new()
    }

    /// Adds a chain to the registry.
    pub fn add_chain(&mut self, chain: Box<dyn Chain>) {
        self.chains.insert(chain.chain_id(), chain);
    }

    /// Removes a chain from the registry.
    pub fn remove_chain(&mut self, id: u64) -> Option<Box<dyn Chain>> {
        self.chains.remove(&id)
    }
}