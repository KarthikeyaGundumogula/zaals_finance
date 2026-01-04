use anchor_lang::prelude::*;

#[error_code]
pub enum SignerError {
    #[msg("Unauthorized: caller is not the authorized agent")]
    UnauthorizedAgent,

    #[msg("The signer is not the owner of the asset")]
    InvalidAssetOwner,

    #[msg("The signer is not the node operator")]
    InvalidNodeOperator,
    
    #[msg("Address cannot be default or system program")]
    InvalidAddress,

    #[msg("Admin address cannot be the system program")]
    AdminCannotBeSystemProgram,

    #[msg("Beneficiary not found in the Beneficary Array")]
    UnauthorizedBeneficiary,
}
