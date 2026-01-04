use anchor_lang::prelude::*;

use crate::constants::DISPUTE_WINDOW;
use crate::errors::*;
use crate::state::{AuthorityConfig, Vault};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct FinalizeSlashReq<'info> {
    #[account(
        mut,
        address = config.agent @ SignerError::UnauthorizedAgent
    )]
    pub agent: Signer<'info>,
    #[account(
        mut,
        seeds = [b"Vault", vault.node_operator.key().as_ref()],
        bump = vault.bump,
        constraint = vault.is_dispute_active @ VaultError::VaultNotUnderDispute
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        seeds = [b"Config"],
        bump = config.bump,
    )]
    pub config: Account<'info, AuthorityConfig>,
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

    /// Slash claimant's token account
    #[account(
        mut,
        associated_token::mint = locking_token_mint,
        associated_token::authority = vault.slash_claimant,
        associated_token::token_program = token_program
    )]
    pub slash_claimant_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FinalizeSlashReq<'info> {
    pub fn process_req(&mut self, decision: bool, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        if decision == true
            && amount <= self.vault.pending_slash_amount
            && amount > 0
            && clock.unix_timestamp < self.vault.dispute_start_time + DISPUTE_WINDOW
        {
            msg!("Processing withdrawl");
            self.process_withdrawal(amount)?;
        }
        self.vault.pending_slash_amount = 0;
        self.vault.dispute_start_time = 0;
        self.vault.is_dispute_active = false;
        Ok(())
    }

    fn process_withdrawal(&mut self, amount: u64) -> Result<()> {
        // Calculate new total vault capital
        let new_slashed_capital = self
            .vault
            .capital_after_slashing
            .checked_sub(amount)
            .ok_or(ArithmeticError::ArithmeticUnderflow)?;

        require_gte!(
            new_slashed_capital,
            self.vault.min_cap,
            VaultError::VaultReachedMinCap
        );

        self.vault.capital_after_slashing = new_slashed_capital;

        let node_operator_key = self.vault.node_operator.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"Vault", node_operator_key.as_ref(), &[self.vault.bump]]];

        let transfer_accounts = TransferChecked {
            from: self.vault_token_ata.to_account_info(),
            to: self.slash_claimant_ata.to_account_info(),
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
