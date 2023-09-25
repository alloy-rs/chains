//! Chain trait.

/// The base Chain trait, from which all representations of a chain are derived.
pub trait Chain {
    /// The chain ID.
    fn chain_id(&self) -> u64;

    /// The chain name.
    fn name(&self) -> &str;
}

/// Metadata about a [Chain] concerning EIP and EVM feature support.
pub trait ChainMetadata {
    /// Whether the chain supports EIP 1559 or not.
    fn is_legacy(&self) -> bool;
    /// Whether the chain supports push0 or not.
    fn supports_push0(&self) -> bool;
}
