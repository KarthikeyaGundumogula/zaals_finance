use anchor_lang::prelude::*;

#[event]
pub struct ProgramInitializedEvent {
    pub config: Pubkey,
    pub admin: Pubkey,
    pub agent: Pubkey,
    pub nft_program: Pubkey,
    pub capital_program: Pubkey,
    pub early_unlock_fee: u64,
    pub dispute_window: i64,
    pub min_lock_duration: i64,
    pub max_lock_duration: i64,
    pub timestamp: i64,
}

#[event]
pub struct VaultCreatedEvent {
    pub vault: Pubkey,
    pub provider: Pubkey,
    pub node_operator: Pubkey,
    pub staking_token: Pubkey,
    pub reward_token: Pubkey,
    pub nft_collection: Pubkey,
    pub min_cap: u64,
    pub max_cap: u64,
    pub lock_phase_start_time: i64,
    pub timestamp: i64,
}

#[event]
pub struct PositionOpenedEvent {
    pub position: Pubkey,
    pub vault: Pubkey,
    pub capital_provider: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct PositionUpdatedEvent {
    pub position: Pubkey,
    pub vault: Pubkey,
    pub capital_provider: Pubkey,
    pub update_amount: i64,
    pub new_total: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsDepositedEvent {
    pub vault: Pubkey,
    pub agent: Pubkey,
    pub amount: u64,
    pub total_rewards: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsClaimedEvent {
    pub holder: Pubkey,
    pub vault: Pubkey,
    pub position: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct OperatorRewardsClaimedEvent {
    pub operator: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct BeneficiaryRewardsClaimedEvent {
    pub beneficiary: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct SlashRequestCreatedEvent {
    pub vault: Pubkey,
    pub agent: Pubkey,
    pub slash_claimant: Pubkey,
    pub slash_bps: u16,
    pub dispute_start_time: i64,
    pub timestamp: i64,
}

#[event]
pub struct SlashReqFinalizedEvent {
    pub claimant: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct PositionClosedEvent {
    pub holder: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct VaultClosedEvent {
    pub node_operator: Pubkey,
    pub timestamp: i64,
}
