use crate::constants::BASE_BPS;
use crate::state::Vault;
use anchor_lang::prelude::*;

use crate::errors::*;

use crate::state::AuthorityConfig;

#[derive(Accounts)]
pub struct CreateSlashReq<'info> {
    /// CHECK: Here agent is a trusted multi-sig or a DAAO so the claimant needs no validation
    /// and claimant can be anyone either one of benificiaries or some DePIN network outside solana
    /// STILL WOKRING ON TRUST-MINIMIZING OPTIONS - DAAO's or Oracle Netwroks or a MULTI-SIG
    #[account(
        mut,
        address = config.agent @ SignerError::UnauthorizedAgent
    )]
    pub agent: Signer<'info>,
    #[account(
        mut,
        seeds = [b"Vault", vault.node_operator.key().as_ref()],
        bump = vault.bump,
        constraint = !vault.is_dispute_active @ VaultError::VaultUnderDispute
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        seeds = [b"Config"],
        bump = config.bump,
    )]
    pub config: Account<'info, AuthorityConfig>,
}

impl<'info> CreateSlashReq<'info> {
    pub fn create_slas_req(&mut self, slash_bps: u16, slash_claimant: Pubkey) -> Result<()> {
        let clock = Clock::get()?;
        require_gte!(
            self.vault.max_slash_bps,
            slash_bps,
            VaultError::SlashReqExceedsMaxBps
        );
        require!(
            clock.unix_timestamp > self.vault.lock_phase_start_at
                && clock.unix_timestamp
                    < self.vault.lock_phase_start_at + self.vault.lock_phase_duration,
            PhaseError::InvalidPhase
        );
        let slash_amount = (slash_bps as u64)
            .checked_mul(self.vault.total_capital_collected)
            .ok_or(ArithmeticError::ArithmeticOverflow)?
            .checked_div(BASE_BPS as u64)
            .ok_or(ArithmeticError::DivisionByZero)?;
        self.vault.is_dispute_active = true;
        self.vault.pending_slash_amount = slash_amount;
        self.vault.slash_claimant = slash_claimant;
        let clock = Clock::get()?;
        self.vault.dispute_start_time = clock.unix_timestamp;
        Ok(())
    }
}
