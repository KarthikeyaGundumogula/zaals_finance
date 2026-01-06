use crate::constants::BASE_BPS;
use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use mpl_core::accounts::BaseAssetV1;

#[derive(Accounts)]
pub struct ClaimInvestorRewards<'info> {
    /// The holder must be the owner of the asset (NFT)
    #[account(
        address = asset.owner @ SignerError::InvalidAssetOwner
    )]
    pub holder: Signer<'info>,

    /// Global configuration account
    #[account(
        seeds = [b"Config"],
        bump = config.bump
    )]
    pub config: Account<'info, AuthorityConfig>,

    /// Vault account holding the pooled capital and rewards
    #[account(
        seeds = [b"Vault", vault.node_operator.key().as_ref()],
        bump = vault.bump,
        constraint = !vault.is_dispute_active @ VaultError::VaultUnderDispute,
    )]
    pub vault: Account<'info, Vault>,

    /// Position account tracking the holder's investment
    #[account(
        mut,
        seeds = [b"Position", asset.key().as_ref()],
        bump = position.bump,
        constraint = position.vault == vault.key() @ PositionError::PositionVaultMismatch
    )]
    pub position: Account<'info, Position>,

    /// The MPL Core asset (NFT) representing the position
    /// CHECK: Validated by position.asset and holder ownership
    #[account(
        address = position.asset @ PositionError::InvalidAsset
    )]
    pub asset: Account<'info, BaseAssetV1>,

    /// The reward token mint
    #[account(
        address = vault.reward_token_mint @ TokenError::InvalidRewardMint,
        mint::token_program = token_program
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    /// Vault's token account holding rewards
    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
        constraint = vault_ata.amount > 0 @ TokenError::InsufficientVaultBalance
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    /// Holder's token account to receive rewards
    #[account(
      mut,
        associated_token::mint = reward_mint,
        associated_token::authority = holder,
        associated_token::token_program = token_program
    )]
    pub holder_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimInvestorRewards<'info> {
    /// Calculates pending rewards for this position
    ///
    /// Formula:
    /// 1. Total investor rewards = total_rewards_deposited * investor_bps / BASE_BPS
    /// 2. Position share = investor_rewards * position_capital / total_vault_capital
    /// 3. Claimable = position_share - already_claimed
    pub fn calculate_claimable_rewards(&self) -> Result<u64> {
        let total_vault_capital = self.vault.total_capital_collected;
        let position_locked_capital = self.position.total_value_locked;
        let total_rewards_deposited = self.vault.total_rewards_deposited;
        let investors_share_bps = self.vault.investor_bps as u64;

        // Preconditions
        require_gt!(total_vault_capital, 0, TokenError::InsufficientVaultBalance);

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

    /// Updates the position state with claimed rewards
    pub fn process_claim(&mut self, amount: u64) -> Result<()> {
        // Validate amount is greater than zero
        require_gt!(amount, 0, ArithmeticError::AmountMustBePositive);

        // Validate vault has sufficient balance
        require_gte!(
            self.vault_ata.amount,
            amount,
            TokenError::InsufficientVaultBalance
        );

        // Update position state
        self.position.total_rewards_claimed = self
            .position
            .total_rewards_claimed
            .checked_add(amount)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        Ok(())
    }

    /// Transfers rewards from vault to holder
    pub fn transfer_rewards(&self, amount: u64) -> Result<()> {
        let node_operator_key = self.vault.node_operator.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"Vault", node_operator_key.as_ref(), &[self.vault.bump]]];

        let transfer_accounts = TransferChecked {
            from: self.vault_ata.to_account_info(),
            to: self.holder_ata.to_account_info(),
            authority: self.vault.to_account_info(),
            mint: self.reward_mint.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, amount, self.reward_mint.decimals)?;

        Ok(())
    }
}
