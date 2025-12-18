use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::errors::*;
use crate::state::*;

use nft_program::cpi::accounts::CreateAsset;
use nft_program::instructions::CreateAssetArgs;
use nft_program::program::NftProgram;
use nft_program::state::NFTConfig;

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    /// The capital provider who is opening the position
    #[account(mut)]
    pub capital_provider: Signer<'info>,

    /// The NFT asset representing this position
    #[account(mut)]
    pub asset: Signer<'info>,

    /// The vault's NFT collection
    /// CHECK: Validated by MPL Core program during CPI
    #[account(
        constraint = vault_collection.key() == vault.nft_collection @ PositionError::InvalidCollection
    )]
    pub vault_collection: UncheckedAccount<'info>,

    /// The vault where capital will be locked
    #[account(
        mut,
        seeds = [b"Vault", vault.node_operator.key().as_ref()],
        bump = vault.bump,
        constraint = !vault.is_dispute_active @ VaultError::VaultUnderDispute
    )]
    pub vault: Account<'info, Vault>,

    /// Global configuration account
    #[account(
        seeds = [b"Config"],
        bump = config.bump
    )]
    pub config: Account<'info, AuthorityConfig>,

    /// NFT Program configuration
    pub nft_config: Account<'info, NFTConfig>,

    /// Position account tracking the investment
    #[account(
        init,
        payer = capital_provider,
        space = Position::INIT_SPACE + 8,
        seeds = [b"Position", asset.key().as_ref()],
        bump,
    )]
    pub position: Account<'info, Position>,

    /// Capital provider's token account
    #[account(
        mut,
        associated_token::mint = locked_token_mint,
        associated_token::authority = capital_provider,
        associated_token::token_program = token_program,
        constraint = capital_provider_token_ata.amount >= vault.min_lock_amount @ TokenError::InsufficientBalance
    )]
    pub capital_provider_token_ata: InterfaceAccount<'info, TokenAccount>,

    /// Vault's token account
    #[account(
        init_if_needed,
        payer = capital_provider,
        associated_token::mint = locked_token_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    /// The token mint for locked capital
    #[account(
        mint::token_program = token_program,
        address = vault.locking_token_mint @ TokenError::InvalidLockingMint
    )]
    pub locked_token_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,

    /// CHECK: Validated by NFT program during CPI
    #[account(executable)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub nft_program: Program<'info, NftProgram>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> OpenPosition<'info> {
    /// Validates position opening parameters
    fn validate_position(&self, amount: u64) -> Result<()> {
        // Validate amount meets minimum
        require_gte!(
            amount,
            self.vault.min_lock_amount,
            VaultError::BelowMinLockCap
        );

        // Validate amount is positive
        require_gt!(amount, 0, ArithmeticError::AmountMustBePositive);

        // Calculate new total after deposit
        let new_total = self
            .vault
            .total_capital_collected
            .checked_add(amount)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        // Validate vault capacity with detailed error
        if new_total > self.vault.max_cap {
            // Check if partial deposit is possible
            let remaining_capacity = self
                .vault
                .max_cap
                .checked_sub(self.vault.total_capital_collected)
                .ok_or(ArithmeticError::ArithmeticUnderflow)?;

            require!(
                remaining_capacity >= self.vault.min_lock_amount,
                VaultError::VaultMaxCapReached
            );

            return err!(VaultError::VaultMaxCapReached);
        }

        // Validate timing constraints
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp < self.vault.lock_phase_start_at,
            PhaseError::LockPhaseAlreadyStarted
        );

        Ok(())
    }

    /// Initializes the position account
    pub fn initialize_position(&mut self, amount: u64, bumps: &OpenPositionBumps) -> Result<()> {
        self.validate_position(amount)?;

        self.position.set_inner(Position {
            vault: self.vault.key(),
            total_value_locked: amount,
            total_rewards_claimed: 0,
            asset: self.asset.key(),
            bump: bumps.position,
        });

        Ok(())
    }

    /// Transfers capital from provider to vault
    pub fn transfer_capital(&mut self, amount: u64) -> Result<()> {
        // Update vault state first (checks before effects pattern)
        self.vault.total_capital_collected = self
            .vault
            .total_capital_collected
            .checked_add(amount)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;
        // Deposits only allowed before Locking and the slashing happens after locking
        self.vault.capital_after_slashing = self.vault.total_capital_collected;

        // Perform the transfer
        let transfer_accounts = TransferChecked {
            from: self.capital_provider_token_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.capital_provider.to_account_info(),
            mint: self.locked_token_mint.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, amount, self.locked_token_mint.decimals)?;

        Ok(())
    }

    /// Mints the NFT asset representing this position
    pub fn mint_position_nft(&self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[b"Config", &[self.config.bump]]];

        let cpi_accounts = CreateAsset {
            asset: self.asset.to_account_info(),
            payer: self.capital_provider.to_account_info(),
            owner: self.capital_provider.to_account_info(),
            system_program: self.system_program.to_account_info(),
            mpl_core_program: self.mpl_core_program.to_account_info(),
            collection: self.vault_collection.to_account_info(),
            config: self.nft_config.to_account_info(),
            collection_update_authority: self.config.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.nft_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        // Create dynamic NFT metadata based on position
        let args = CreateAssetArgs {
            name: format!(
                "Vault Position #{}",
                self.position.key().to_string()[..8].to_string()
            ),
            uri: format!("https://api.vault.com/position/{}", self.position.key()),
        };

        nft_program::cpi::create_core_asset_handler(cpi_ctx, args)?;

        Ok(())
    }
}
