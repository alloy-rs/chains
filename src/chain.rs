//! Chain struct, covering all networks.

/// The base Chain struct, from which all representations of a chain are derived.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chain {
    /// The chain ID.
    pub id: u64,
    /// The name of the chain.
    pub name: &'static str,
}

impl Chain {
    /// Gets the chain ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the chain name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Creates one or more [Chain]s.
///
/// This macro allows for quick and convenient creation of [Chain] definitions at compile time.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate my_lib;
/// # fn main() {
///     create_chains!(
///         MAINNET, 1, "Ethereum";
///         SEPOLIA, 11155111, "Sepolia"
///     );
/// 
/// println!("Ethereum: {}, {}", MAINNET.id, MAINNET.name);
/// println!("Sepolia: {}, {}", SEPOLIA.id, SEPOLIA.name);
/// # }
/// ```

#[macro_export]
macro_rules! chains {
    ($($name:ident, $id:expr, $chain_name:expr);*) => {
        $(
            const $name: Chain = Chain { id: $id, name: $chain_name };
        )*
    };
}