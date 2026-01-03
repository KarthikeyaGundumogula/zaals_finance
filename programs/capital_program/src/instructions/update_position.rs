use crate::{
    errors::*,
    state::{AuthorityConfig, Position, Vault},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use mpl_core::accounts::BaseAssetV1;

#[derive(Accounts)]
pub struct UpdatePosition<'info> {
    /// The capital provider who owns the position
    #[account(
        mut,
        address = asset.owner @ SignerError::InvalidAssetOwner
    )]
    pub capital_provider: Signer<'info>,

    /// The vault containing this position
    #[account(
        mut,
        seeds = [b"Vault", vault.node_operator.key().as_ref()],
        bump = vault.bump,
        constraint = !vault.is_dispute_active @ VaultError::VaultUnderDispute
    )]
    pub vault: Account<'info, Vault>,

    /// Global configuration
    #[account(
        seeds = [b"Config"],
        bump = config.bump
    )]
    pub config: Account<'info, AuthorityConfig>,

    /// The position being updated
    #[account(
        mut,
        seeds = [b"Position", asset.key().as_ref()],
        bump = position.bump,
        constraint = position.vault == vault.key() @ PositionError::PositionVaultMismatch,
    )]
    pub position: Account<'info, Position>,

    /// The NFT asset representing the position
    /// CHECK: Validated by position.asset and capital_provider ownership
    #[account(
        address = position.asset @ PositionError::InvalidAsset
    )]
    pub asset: Account<'info, BaseAssetV1>,

    /// Locking token mint
    #[account(
        mint::token_program = token_program,
        address = vault.locking_token_mint @ TokenError::InvalidLockingMint
    )]
    pub locking_token_mint: InterfaceAccount<'info, Mint>,

    /// Vault's token account
    #[account(
        mut,
        associated_token::mint = locking_token_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_token_ata: InterfaceAccount<'info, TokenAccount>,

    /// Capital provider's token account
    #[account(
        mut,
        associated_token::mint = locking_token_mint,
        associated_token::authority = capital_provider,
        associated_token::token_program = token_program
    )]
    pub capital_provider_token_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdatePosition<'info> {
    /// Validates and processes position update (deposit or withdrawal)
    pub fn process_update(&mut self, update_amount: i64) -> Result<()> {
        let clock = Clock::get()?;

        // Validate timing - can't update during lock phase
        require!(
            clock.unix_timestamp < self.vault.lock_phase_start_at,
            PhaseError::LockPhaseAlreadyStarted
        );

        if update_amount > 0 {
            // Deposit additional capital
            self.process_deposit(update_amount as u64)?;
        } else if update_amount < 0 {
            // Withdraw capital
            self.process_withdrawal((-update_amount) as u64)?;
        } else {
            return err!(ArithmeticError::UpdateAmountCannotBeZero);
        }

        Ok(())
    }

    /// Processes capital deposit (increase position)
    fn process_deposit(&mut self, amount: u64) -> Result<()> {
        // Validate amount is positive
        require_gt!(amount, 0, ArithmeticError::AmountMustBePositive);

        // Calculate new position value
        let new_position_value = self
            .position
            .total_value_locked
            .checked_add(amount)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        // Calculate new total vault capital
        let new_total_capital = self
            .vault
            .total_capital_collected
            .checked_add(amount)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        // Validate vault capacity
        require_gte!(
            self.vault.max_cap,
            new_total_capital,
            VaultError::VaultMaxCapReached
        );

        // Validate provider has sufficient balance
        require_gte!(
            self.capital_provider_token_ata.amount,
            amount,
            TokenError::InsufficientBalance
        );

        // Update state
        self.position.total_value_locked = new_position_value;
        self.vault.total_capital_collected = new_total_capital;
        self.vault.capital_after_slashing = new_total_capital;

        // Transfer tokens
        self.transfer_to_vault(amount)?;

        Ok(())
    }

    /// Processes capital withdrawal (decrease position)
    fn process_withdrawal(&mut self, amount: u64) -> Result<()> {
        // Validate amount is positive
        require_gt!(amount, 0, ArithmeticError::AmountMustBePositive);

        // Calculate new position value
        let new_position_value = self
            .position
            .total_value_locked
            .checked_sub(amount)
            .ok_or(ArithmeticError::ArithmeticUnderflow)?;

        // Calculate new total vault capital
        let new_total_capital = self
            .vault
            .total_capital_collected
            .checked_sub(amount)
            .ok_or(ArithmeticError::ArithmeticUnderflow)?;

        // Validate new position meets minimum requirement
        require_gte!(
            new_position_value,
            self.vault.min_lock_amount,
            ArithmeticError::AmountBelowMinimum
        );

        // Validate vault has sufficient balance
        require_gte!(
            self.vault_token_ata.amount,
            amount,
            TokenError::InsufficientVaultBalance
        );

        // Update state
        self.position.total_value_locked = new_position_value;
        self.vault.total_capital_collected = new_total_capital;
        self.vault.capital_after_slashing = new_total_capital;

        // Transfer tokens
        self.transfer_from_vault(amount)?;

        Ok(())
    }

    /// Transfers tokens from provider to vault
    fn transfer_to_vault(&self, amount: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.capital_provider_token_ata.to_account_info(),
            to: self.vault_token_ata.to_account_info(),
            authority: self.capital_provider.to_account_info(),
            mint: self.locking_token_mint.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, amount, self.locking_token_mint.decimals)?;

        Ok(())
    }

    /// Transfers tokens from vault to provider
    fn transfer_from_vault(&self, amount: u64) -> Result<()> {
        let node_operator_key = self.vault.node_operator.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"Vault", node_operator_key.as_ref(), &[self.vault.bump]]];

        let transfer_accounts = TransferChecked {
            from: self.vault_token_ata.to_account_info(),
            to: self.capital_provider_token_ata.to_account_info(),
            authority: self.vault.to_account_info(),
            mint: self.locking_token_mint.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, amount, self.locking_token_mint.decimals)?;

        Ok(())
    }
}
