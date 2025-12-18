use crate::constants::*;
use crate::errors::*;
use crate::state::{AuthorityConfig, Beneficiary, Vault};
use nft_program::cpi::accounts::CreateVaultCollection;
use nft_program::program::NftProgram;
use nft_program::state::NFTConfig;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenInterface},
};

#[derive(Accounts)]
#[instruction(config: InitVaultConfig)]
pub struct CreateVault<'info> {
    /// The vault provider/creator who pays for account initialization
    #[account(mut)]
    pub provider: Signer<'info>,

    /// The vault account to be created
    #[account(
        init,
        payer = provider,
        seeds = [b"Vault", provider.key().as_ref()],
        space = Vault::INIT_SPACE + 8,
        bump,
    )]
    pub vault: Account<'info, Vault>,

    /// Global authority configuration
    #[account(
        seeds = [b"Config"],
        bump = config_account.bump,
    )]
    pub config_account: Account<'info, AuthorityConfig>,

    /// NFT marketplace configuration
    pub nft_config: Account<'info, NFTConfig>,

    /// Reward token mint - tokens distributed as rewards
    #[account(
        mint::token_program = token_program,
        constraint = reward_token_mint.decimals > 0 @ TokenError::InvalidRewardMint
    )]
    pub reward_token_mint: InterfaceAccount<'info, Mint>,

    /// Staking/locking token mint - tokens locked by investors
    #[account(
        mint::token_program = token_program,
        constraint = lock_mint.decimals > 0 @ TokenError::InvalidLockingMint,
    )]
    pub lock_mint: InterfaceAccount<'info, Mint>,

    /// NFT collection for vault positions
    #[account(
        mut,
        constraint = nft_collection.lamports() == 0 @ NFTProgramError::CollectionAlreadyExists
    )]
    pub nft_collection: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Validated by MPL Core program during CPI
    #[account(executable)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub nft_marketplace: Program<'info, NftProgram>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateVault<'info> {
    /// Validates all configuration parameters for vault creation
    ///
    /// Checks:
    /// - Total BPS allocation doesn't exceed 100%
    /// - Lock phase duration meets minimum requirements
    /// - Capital caps are properly ordered
    /// - Timing constraints are satisfied
    /// - Beneficiary configuration is valid
    pub fn validate_config(&self, config: &InitVaultConfig) -> Result<()> {
        // Validate no duplicate beneficiaries and calculate total BPS
        let mut total_beneficiary_bps: u16 = 0;
        for i in 0..config.beneficiaries.len() {
            // Check for duplicates
            for j in (i + 1)..config.beneficiaries.len() {
                require_neq!(
                    config.beneficiaries[i].address,
                    config.beneficiaries[j].address,
                    VaultError::DuplicateBeneficiary
                );
            }

            // Validate individual share is reasonable
            require_gt!(
                config.beneficiaries[i].share_bps,
                0,
                VaultError::BeneficiaryShareMustBePositive
            );

            // Accumulate total
            total_beneficiary_bps = total_beneficiary_bps
                .checked_add(config.beneficiaries[i].share_bps)
                .ok_or(ArithmeticError::ArithmeticOverflow)?;
        }

        // Validate total BPS doesn't exceed 100%
        let total_bps = total_beneficiary_bps
            .checked_add(config.investor_bps)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        require_gte!(BASE_BPS, total_bps, VaultError::BPSExceedsMaximum);

        // Validate lock phase duration
        require_gte!(
            config.lock_phase_duration,
            MIN_LOCK_PERIOD,
            PhaseError::LockPhaseTooShort
        );

        // Validate capital caps (max should be greater than min)
        require_gt!(
            config.max_cap,
            config.min_cap,
            VaultError::InvalidCapitalRange
        );

        require_gt!(config.min_cap, 0, VaultError::MinCapMustBePositive);

        require_gt!(
            config.min_lock_amount,
            0,
            VaultError::MinLockAmountMustBePositive
        );

        // Validate timing constraints
        let clock = Clock::get()?;
        let earliest_lock_time = clock
            .unix_timestamp
            .checked_add(MIN_FUND_RAISE_DURATION)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        require_gte!(
            config.lock_phase_start_time,
            earliest_lock_time,
            PhaseError::LockPhaseStartsTooSoon
        );

        Ok(())
    }

    /// Initializes the vault account with provided configuration
    pub fn initialize_vault(
        &mut self,
        config: InitVaultConfig,
        bumps: &CreateVaultBumps,
    ) -> Result<()> {
        self.vault.set_inner(Vault {
            // Token configuration
            locking_token_mint: self.lock_mint.key(),
            reward_token_mint: self.reward_token_mint.key(),

            // Capital configuration
            min_cap: config.min_cap,
            max_cap: config.max_cap,
            min_lock_amount: config.min_lock_amount,
            total_capital_collected: 0,
            total_rewards_deposited: 0,
            capital_after_slashing: 0,

            // Beneficiary configuration
            beneficiaries: config.beneficiaries,
            investor_bps: config.investor_bps,

            // Slash configuration
            max_slash_bps: config.max_slash_bps,
            pending_slash_amount: 0,
            slash_claimant: config.slash_claimant,

            // NFT configuration
            nft_collection: self.nft_collection.key(),

            // Authority configuration
            reward_distributor: config.reward_distributor,
            node_operator: config.node_operator,

            // Timing configuration
            lock_phase_start_at: config.lock_phase_start_time,
            lock_phase_duration: config.lock_phase_duration,

            is_dispute_active: false,
            dispute_start_time: 0,

            // Account metadata
            bump: bumps.vault,
        });

        Ok(())
    }

    /// Creates the NFT collection for this vault via CPI
    pub fn create_nft_collection(&self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[b"Config", &[self.config_account.bump]]];

        let cpi_accounts = CreateVaultCollection {
            collection: self.nft_collection.to_account_info(),
            update_authority: self.config_account.to_account_info(),
            config: self.nft_config.to_account_info(),
            payer: self.provider.to_account_info(),
            mpl_core_program: self.mpl_core_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.nft_marketplace.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        nft_program::cpi::create_vault_collection_handler(cpi_ctx)?;

        Ok(())
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitVaultConfig {
    // Capital configuration
    pub min_cap: u64,
    pub max_cap: u64,
    pub min_lock_amount: u64,

    // Beneficiary configuration
    pub beneficiaries: Vec<Beneficiary>,
    pub investor_bps: u16,

    // Slash configuration
    pub max_slash_bps: u16,
    pub slash_claimant: Pubkey,

    // Authority configuration
    pub reward_distributor: Pubkey,
    pub node_operator: Pubkey,

    // Timing configuration
    pub lock_phase_duration: i64,
    pub lock_phase_start_time: i64,
}
