use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Total basis points allocation exceeds maximum (10000 BPS = 100%)")]
    BPSExceedsMaximum,

    #[msg("Slash basis points exceeds maximum allowed")]
    SlashBPSExceedsMaximum,

    #[msg("Number of beneficiaries does not match number of share allocations")]
    BeneficiaryMismatch,

    #[msg("Too many beneficiaries specified (exceeds maximum limit)")]
    TooManyBeneficiaries,

    #[msg("Duplicate beneficiary address detected")]
    DuplicateBeneficiary,

    #[msg("Invalid basis points configuration")]
    InvalidBasisPoints,

    #[msg("Beneficiary shares must sum to allocated BPS")]
    BeneficiarySharesMismatch,

    #[msg("Maximum cap must be greater than minimum cap")]
    InvalidCapitalRange,

    #[msg("Minimum cap must be greater than zero")]
    MinCapMustBePositive,

    #[msg("Minimum lock amount must be greater than zero")]
    MinLockAmountMustBePositive,

    #[msg("Amount Must be greater than the Min Lock Amount")]
    BelowMinLockCap,

    #[msg("Lock Duration must be above MIN_LOCK_DURATION")]
    LockDurationRangeTooNarrow,

    #[msg("Beneficiary shares must greater than zero")]
    BeneficiaryShareMustBePositive,

    #[msg("The vault is currently under dispute and claims are disabled")]
    VaultUnderDispute,

    #[msg("Slash req is more than vault configured slash req limit")]
    SlashReqExceedsMaxBps,

    #[msg("Vault has reached maximum capacity")]
    VaultMaxCapReached,

    #[msg("Vault has no capital collected")]
    NoCapitalInVault,

    #[msg("Given Index is out of Beneficiary")]
    InvalidBeneficiaryIndex,

    #[msg("Vault still has some locked tokens in it")]
    VaultNotEmpty,

    #[msg("Vault is not under dispute or claim_request is expired")]
    VaultNotUnderDispute,

    #[msg("Vault doesn't reach the min cap")]
    BelowMinCap,

    #[msg("After this operation vault reaches less than MIN_CAP")]
    VaultReachedMinCap,
}
