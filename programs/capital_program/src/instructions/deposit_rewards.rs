use crate::errors::*;
use crate::state::{AuthorityConfig, Vault};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct DepositRewards<'info> {
    /// The authorized agent depositing rewards
    #[account(
        mut,
        address = config.agent @ SignerError::UnauthorizedAgent
    )]
    pub agent: Signer<'info>,

    /// The vault receiving rewards
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
        bump = config.bump,
    )]
    pub config: Account<'info, AuthorityConfig>,

    /// Reward token mint
    #[account(
        mint::token_program = token_program,
        address = vault.reward_token_mint @ TokenError::InvalidRewardMint
    )]
    pub reward_token_mint: InterfaceAccount<'info, Mint>,

    /// Vault's reward token account
    #[account(
        init_if_needed,
        payer = agent,
        associated_token::mint = reward_token_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_reward_ata: InterfaceAccount<'info, TokenAccount>,

    /// Agent's reward token account
    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = agent,
        associated_token::token_program = token_program,
        constraint = agent_reward_ata.amount > 0 @ TokenError::InsufficientBalance
    )]
    pub agent_reward_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositRewards<'info> {
    /// Validates reward deposit parameters
    pub fn validate_deposit(&self, amount: u64) -> Result<()> {
        // Validate amount is positive
        require_gt!(amount, 0, ArithmeticError::AmountMustBePositive);

        // Validate agent has sufficient balance
        require_gte!(
            self.agent_reward_ata.amount,
            amount,
            TokenError::InsufficientBalance
        );

        // Validate vault has capital collected
        require_gt!(
            self.vault.total_capital_collected,
            self.vault.min_cap,
            VaultError::BelowMinCap
        );

        // Validate timing - can only deposit rewards after lock phase starts
        let clock = Clock::get()?;
        msg!("Current time: {}", clock.unix_timestamp);
        require!(
            clock.unix_timestamp >= self.vault.lock_phase_start_at,
            PhaseError::InvalidPhase
        );
        Ok(())
    }

    /// Updates vault state with new reward deposit
    pub fn update_vault_state(&mut self, amount: u64) -> Result<()> {
        let new_total_rewards = self
            .vault
            .total_rewards_deposited
            .checked_add(amount)
            .ok_or(ArithmeticError::ArithmeticOverflow)?;

        self.vault.total_rewards_deposited = new_total_rewards;

        Ok(())
    }

    /// Transfers reward tokens from agent to vault
    pub fn transfer_rewards(&self, amount: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.agent_reward_ata.to_account_info(),
            to: self.vault_reward_ata.to_account_info(),
            authority: self.agent.to_account_info(),
            mint: self.reward_token_mint.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, amount, self.reward_token_mint.decimals)?;

        Ok(())
    }
}
