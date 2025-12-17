#![allow(unused_imports)]

pub(crate) mod generated;

pub use generated::capital_program::*;

pub mod nft_program {
    pub use super::generated::nft_program::*;
  }

pub mod capital_program {
    pub use super::generated::capital_program::*;
}

pub use generated::capital_program::programs::CAPITAL_PROGRAM_ID;
pub use generated::nft_program::programs::NFT_PROGRAM_ID;
