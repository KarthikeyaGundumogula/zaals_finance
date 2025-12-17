use anchor_lang::prelude::*;

#[error_code]
pub enum ExteranlProgramError {
    #[msg("mpl_core_program passed in accounts didn't match")]
    InvalidMPLCoreProgramId,
}

#[error_code]
pub enum SignerError {
    #[msg("Signer must be the owner or delgator of the asset")]
    InvalidAssetOwner,
}

#[error_code]
pub enum OfferError {
    #[msg("Mint does not match seller listed token mint")]
    InvalidMint,
    #[msg("Seller must be the one in the offer")]
    InvalidSeller,
}
