use anchor_lang::prelude::*;

#[error_code]
pub enum PositionError {
    #[msg("Invalid asset address")]
    InvalidAsset,

    #[msg("Position does not belong to this vault")]
    PositionVaultMismatch,

    #[msg("Invalid collection address")]
    InvalidCollection,

    #[msg("Position calimed all the rewards or no rewards are accumulated")]
    NoRewardsToClaim,

    #[msg("There Position still has some unclaimed rewards or capital")]
    PositionIsNotEmpty,
}
