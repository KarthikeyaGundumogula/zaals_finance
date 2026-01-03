use anchor_lang::prelude::*;
use nft_program::program::NftProgram;

use crate::constants::*;
use crate::errors::*;
use crate::state::AuthorityConfig;

#[derive(Accounts)]
#[instruction(params: InitProgramConfig)]
pub struct InitProgram<'info> {
    /// Global authority configuration account
    /// re-initialization is not possible because of the init constraint
    #[account(
        init,
        payer = admin,
        seeds = [b"Config"],
        space = AuthorityConfig::INIT_SPACE + 8,
        bump,
    )]
    pub config: Account<'info, AuthorityConfig>,

    /// Program administrator with initialization authority
    #[account(mut)]
    pub admin: Signer<'info>,

    /// NFT marketplace program
    #[account(
        executable,
        constraint = nft_program.key() == nft_program::ID @ NFTProgramError::InvalidNFTProgram
    )]
    pub nft_program: Program<'info, NftProgram>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitProgram<'info> {
    pub fn validate_params(&self, params: &InitProgramConfig) -> Result<()> {
        // Validate agent address
        require_keys_neq!(params.agent, Pubkey::default(), SignerError::InvalidAddress);

        // Validate reasonable duration range
        let duration_range = params
            .max_lock_duration
            .checked_sub(params.min_lock_duration)
            .ok_or(ArithmeticError::ArithmeticUnderflow)?;

        require_gte!(
            duration_range,
            MIN_LOCK_PERIOD,
            VaultError::LockDurationRangeTooNarrow
        );

        Ok(())
    }

    /// Initializes the program configuration account
    pub fn initialize_config(
        &mut self,
        params: InitProgramConfig,
        bumps: &InitProgramBumps,
    ) -> Result<()> {
        self.config.set_inner(AuthorityConfig {
            // Program references
            nft_program: self.nft_program.key(),

            // Authority configuration
            admin: self.admin.key(),
            agent: params.agent,

            // Fee configuration
            early_unlock_fee: params.early_unlock_fee,

            min_lock_duration: params.min_lock_duration,
            max_lock_duration: params.max_lock_duration,

            // PDA bump
            bump: bumps.config,
        });

        Ok(())
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitProgramConfig {
    /// Agent address for administrative operations
    pub agent: Pubkey,

    /// Fee charged for early unlock (in lamports or basis points)
    pub early_unlock_fee: u64,

    /// Time window for dispute resolution (in seconds)
    pub dispute_window: i64,

    /// Maximum allowed lock duration (in seconds)
    pub max_lock_duration: i64,

    /// Minimum allowed lock duration (in seconds)
    pub min_lock_duration: i64,
}
