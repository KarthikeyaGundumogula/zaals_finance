use crate::constants::BASE_BPS;
use crate::errors::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use mpl_core::accounts::BaseAssetV1;

use mpl_core::{instructions::BurnV1CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    /// The capital provider who owns the position
    #[account(
        mut,
        address = asset.owner @ SignerError::InvalidAssetOwner
    )]
    pub position_holder: Signer<'info>,

    /// The vault containing this position
    #[account(
        mut,
        close = position_holder,
        seeds = [b"Vault", vault.node_operator.key().as_ref()],
        bump = vault.bump,
        constraint = !vault.is_dispute_active @ VaultError::VaultUnderDispute
    )]
    pub vault: Account<'info, Vault>,

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

    /// The NFT asset representing the position
    /// CHECK: Validated by vault.collection and capital_provider ownership
    #[account(
        address = vault.nft_collection @ PositionError::InvalidCollection
    )]
    pub collection: UncheckedAccount<'info>,

    #[account(
        mint::token_program = token_program,
        address = vault.locking_token_mint @ TokenError::InvalidLockingMint
    )]
    pub lock_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = lock_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_lock_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = lock_mint,
        associated_token::authority = position_holder,
        associated_token::token_program = token_program
    )]
    pub capital_provider_lock_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: this will be cheked at marketplace
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClosePosition<'info> {
    pub fn calculate_claimable_rewards(&self) -> Result<u64> {
        let total_vault_capital = self.vault.total_capital_collected;
        let position_locked_capital = self.position.total_value_locked;
        let total_rewards_deposited = self.vault.total_rewards_deposited;
        let investors_share_bps = self.vault.investor_bps as u64;

        // ---- STEP 1: investor rewards (u128 math) ----
        let rewards_for_investors = (total_rewards_deposited as u128)
            .checked_mul(investors_share_bps as u128)
            .ok_or(ArithmeticError::ArithmeticOverflow)?
            .checked_div(BASE_BPS as u128)
            .ok_or(ArithmeticError::DivisionByZero)?;

        // ---- STEP 2: position share (u128 math) ----
        let position_total_rewards = rewards_for_investors
            .checked_mul(position_locked_capital as u128)
            .ok_or(ArithmeticError::ArithmeticOverflow)?
            .checked_div(total_vault_capital as u128)
            .ok_or(ArithmeticError::DivisionByZero)?;

        // ---- STEP 3: subtract claimed ----
        let claimable = position_total_rewards
            .checked_sub(self.position.total_rewards_claimed as u128)
            .ok_or(ArithmeticError::ArithmeticUnderflow)?;

        // ---- STEP 4: back to u64 ----
        let claimable: u64 = claimable
            .try_into()
            .map_err(|_| ArithmeticError::ArithmeticOverflow)?;

        Ok(claimable)
    }

    fn transfer_capital(&self, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.vault_lock_ata.to_account_info(),
            to: self.capital_provider_lock_ata.to_account_info(),
            authority: self.vault.to_account_info(),
            mint: self.lock_mint.to_account_info(),
        };
        let operator = self.vault.node_operator.key();
        let seeds = &[b"Vault", operator.as_ref(), &[self.vault.bump]];
        let signer = &[&seeds[..]];
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer_checked(cpi_ctx, amount, self.lock_mint.decimals)
    }

    pub fn validate_closing_process_unlock(&mut self) -> Result<()> {
        let claimable_rewards = self.calculate_claimable_rewards()?;
        require_gte!(0, claimable_rewards, PositionError::PositionIsNotEmpty);
        let clock = Clock::get()?;
        let lock_starts_at = self.vault.lock_phase_start_at;
        let lock_ends_at = lock_starts_at + self.vault.lock_phase_duration;
        require!(
            clock.unix_timestamp > lock_ends_at
                || (clock.unix_timestamp > lock_starts_at
                    && self.vault.min_cap > self.vault.total_capital_collected),
            PhaseError::InvalidPhase
        );
        let total_capital_collected = self.vault.total_capital_collected;
        let capital_after_slashing = self.vault.capital_after_slashing;
        let mut position_capital = self.position.total_value_locked;
        if total_capital_collected != capital_after_slashing {
            position_capital =
                self.position.total_value_locked * capital_after_slashing / total_capital_collected;
        }
        self.transfer_capital(position_capital)?;
        Ok(())
    }

    pub fn burn_nft(&mut self) -> Result<()> {
        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .payer(&self.position_holder.to_account_info())
            .authority(Some(&self.position_holder.to_account_info()))
            .collection(Some(&self.collection.as_ref()))
            .invoke()?;
        Ok(())
    }
}

#[error_code]
pub enum UnstakeError {
    #[msg("Only owner of the NFT can Unstake")]
    OnlyNFTOwner,
}
