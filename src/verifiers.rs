/// Represents the verification service type.
pub enum VerifierType {
    Etherscan,
    Blockscout,
    Routescan,
    Sourcify,
    Custom(&'static str),
}
