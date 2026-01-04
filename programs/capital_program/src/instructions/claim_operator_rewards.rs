use crate::constants::BASE_BPS;
use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct ClaimOperatorRewards<'info> {
    /// The node operator must be the owner of the vault
    #[account(
        address = vault.node_operator @ SignerError::InvalidNodeOperator,
    )]
    pub node_operator: Signer<'info>,

    /// Vault account holding the pooled capital and rewards
    #[account(
        seeds = [b"Vault", vault.node_operator.key().as_ref()],
        bump = vault.bump,
        constraint = !vault.is_dispute_active @ VaultError::VaultUnderDispute,
    )]
    pub vault: Account<'info, Vault>,

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
        associated_token::authority = node_operator,
        associated_token::token_program = token_program
    )]
    pub operator_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimOperatorRewards<'info> {
    /// Calculates pending rewards for this position
    ///
    /// Formula:
    /// 1. Total Beneficiary BPS
    /// 2. Operator Share = BASE_BPS - Total Beneficiary BPS + Investor Reward BPS
    /// 3. Operator Rewards = (Total Rewards Deposited * Operator Share BPS / BASE_BPS)
    pub fn calculate_claimable_rewards(&self) -> Result<u64> {
        let total_rewards_deposited = self.vault.total_rewards_deposited;
        let total_rewards_claimed = self.vault.operator_rewards_claimed;
        let investors_share_bps = self.vault.investor_bps;
        let beneficiaries = &self.vault.beneficiaries;

        // Calculate total beneficiary BPS
        let total_beneficiary_bps: u16 = beneficiaries.iter().map(|b| b.share_bps).sum();
        // Calculate operator share BPS
        let operator_share_bps = BASE_BPS
            .checked_sub(total_beneficiary_bps)
            .and_then(|bps| bps.checked_sub(investors_share_bps))
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        // Calculate operator rewards
        let operator_rewards = total_rewards_deposited
            .checked_mul(operator_share_bps as u64)
            .and_then(|r| r.checked_div(BASE_BPS as u64))
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        let claimable_rewards = operator_rewards
            .checked_sub(total_rewards_claimed)
            .ok_or(ArithmeticError::ArithmeticUnderflow)?;

        Ok(claimable_rewards)
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

        // update vault state
        self.vault.operator_rewards_claimed = self
            .vault
            .operator_rewards_claimed
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
            to: self.operator_ata.to_account_info(),
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
