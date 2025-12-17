#[path = "capital-program/mod.rs"]
pub(crate) mod capital_program;

#[path = "nft-program/mod.rs"]
pub(crate) mod nft_program;

pub mod types {
    pub use super::capital_program::types::*;
    pub use super::nft_program::types::UpdateAuthority;
}
