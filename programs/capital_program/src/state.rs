use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AuthorityConfig {
    pub nft_program: Pubkey,
    pub admin: Pubkey,
    pub agent: Pubkey,
    pub early_unlock_fee: u64, // in bps to the base 10_000
    pub min_lock_duration: i64,
    pub max_lock_duration: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub locking_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,

    pub min_cap: u64,
    pub max_cap: u64,
    pub min_lock_amount: u64,
    pub total_rewards_deposited: u64,
    pub total_capital_collected: u64,
    pub operator_rewards_claimed: u64,
    pub capital_after_slashing: u64,

    #[max_len(5)]
    pub beneficiaries: Vec<Beneficiary>,
    pub investor_bps: u16,

    pub max_slash_bps: u16,
    pub nft_collection: Pubkey,

    pub node_operator: Pubkey,

    pub lock_phase_start_at: i64,
    pub lock_phase_duration: i64,
    // Slashing
    pub is_dispute_active: bool,
    pub dispute_start_time: i64,
    pub pending_slash_amount: u64,
    pub slash_claimant: Pubkey,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Beneficiary {
    pub address: Pubkey,
    pub share_bps: u16,
    pub total_claimed: u64,
}

#[derive(InitSpace)]
#[account]
pub struct Position {
    pub vault: Pubkey,
    pub asset: Pubkey,
    pub total_value_locked: u64,
    pub total_rewards_claimed: u64,
    pub bump: u8,
}
