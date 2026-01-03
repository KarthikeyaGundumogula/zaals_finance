#![allow(dead_code)]
#![allow(unused)]
use crate::setup::{
    accounts,
    constants::{
        FUND_RAISE_PERIOD, INVESTOR_BPS, MAX_SLASH_BPS, MAX_VAULT_THRESHOLD, MIN_LOCK_AMOUNT,
        MIN_VAULT_TARGET, ONE_DAY,
    },
    test_config::{TestConfig, Tokens},
    *,
};
use anchor_lang::solana_program::example_mocks::solana_transaction::Transaction;
use litesvm::types::TransactionResult;
use solana_sdk::{clock::Clock, signature::Signer};

use zaals_finance_client::{
    capital_program::instructions::{
        ClaimInvestorRewardsHandlerBuilder, CreateVaultHandlerBuilder,
        DepositRewardsHandlerBuilder, InitCapitalProgramHandlerBuilder, OpenPositionHandlerBuilder,
        UpdatePositionHandlerBuilder,
    },
    nft_program::instructions::InitNftProgramHandlerBuilder,
    CAPITAL_PROGRAM_ID, NFT_PROGRAM_ID,
};

pub fn init_nft_program(test_config: &mut TestConfig) -> TransactionResult {
    let config_address = accounts::get_nft_config_pda();
    let authority_config = accounts::get_authority_config_pda();
    let inxs = instruction_handlers::InitNftProgramHandlerBuilder::new()
        .admin(test_config.admin.pubkey())
        .authority(authority_config)
        .capital_program(CAPITAL_PROGRAM_ID)
        .config(config_address)
        .instruction();
    utils::send_transaction(
        &mut test_config.svm,
        &[inxs],
        &test_config.god.pubkey(),
        &[
            &test_config.god.insecure_clone(),
            &test_config.admin.insecure_clone(),
        ],
    )
}

pub fn init_capital_program(test_config: &mut TestConfig) -> TransactionResult {
    let authority_config_address = accounts::get_authority_config_pda();
    let inxs = InitCapitalProgramHandlerBuilder::new()
        .admin(test_config.admin.pubkey())
        .agent(test_config.agent.pubkey())
        .config(authority_config_address)
        .dispute_window(2 * 86400)
        .early_unlock_fee(2_000)
        .max_lock_duration(365 * 86400)
        .min_lock_duration(31 * 86400)
        .nft_program(NFT_PROGRAM_ID)
        .instruction();
    utils::send_transaction(
        &mut test_config.svm,
        &[inxs],
        &test_config.god.pubkey(),
        &[
            &test_config.admin.insecure_clone(),
            &test_config.god.insecure_clone(),
        ],
    )
}

pub fn capital_program_create_vault(
    test_config: &mut TestConfig,
    token_data: &mut Tokens,
) -> TransactionResult {
    let position_vault = accounts::get_vault_pda(test_config.node_operator.pubkey());
    let authority_config = accounts::get_authority_config_pda();
    let nft_config = accounts::get_nft_config_pda();
    let reward_mint = token_data.reward_mint;
    let lock_mint = token_data.lock_mint;
    let collection = token_data.collection.insecure_clone();

    let clock: Clock = test_config.svm.get_sysvar();
    let beneficiaries = test_config.beneficiaries.clone();

    let inxs = CreateVaultHandlerBuilder::new()
        .vault(position_vault)
        .config_account(authority_config)
        .nft_collection(collection.pubkey())
        .nft_config(nft_config)
        .lock_mint(lock_mint)
        .reward_token_mint(reward_mint)
        .node_operator(test_config.node_operator.pubkey())
        .lock_phase_duration(60 * ONE_DAY)
        .lock_phase_start_time(clock.unix_timestamp + FUND_RAISE_PERIOD)
        .max_cap(MAX_VAULT_THRESHOLD)
        .max_slash_bps(MAX_SLASH_BPS)
        .min_cap(MIN_LOCK_AMOUNT)
        .min_lock_amount(MIN_LOCK_AMOUNT)
        .investor_bps(INVESTOR_BPS)
        .beneficiaries(beneficiaries)
        .instruction();

    utils::send_transaction(
        &mut test_config.svm,
        &[inxs],
        &test_config.god.pubkey(),
        &[
            &test_config.node_operator.insecure_clone(),
            &test_config.god.insecure_clone(),
            &collection.insecure_clone(),
        ],
    )
}

pub fn capital_program_open_position(
    test_config: &mut TestConfig,
    token_data: &mut Tokens,
    amount: u64,
) -> TransactionResult {
    let vault_pda = accounts::get_vault_pda(test_config.node_operator.pubkey());
    let authority_config = accounts::get_authority_config_pda();
    let nft_config = accounts::get_nft_config_pda();
    let position_pda = accounts::get_position_pda(token_data.asset.pubkey());
    let inxs = OpenPositionHandlerBuilder::new()
        .capital_provider(test_config.capital_provider.pubkey())
        .asset(token_data.asset.pubkey())
        .vault_collection(token_data.collection.pubkey())
        .vault(vault_pda)
        .config(authority_config)
        .nft_config(nft_config)
        .position(position_pda)
        .capital_provider_token_ata(token_data.provider_lock_ata)
        .vault_ata(token_data.vault_lock_ata)
        .locked_token_mint(token_data.lock_mint)
        .amount(amount)
        .instruction();

    utils::send_transaction(
        &mut test_config.svm,
        &[inxs],
        &test_config.god.pubkey(),
        &[
            &test_config.capital_provider.insecure_clone(),
            &test_config.god.insecure_clone(),
            &token_data.asset.insecure_clone(),
        ],
    )
}

pub fn capital_program_update_position(
    test_config: &mut TestConfig,
    token_data: &mut Tokens,
    update_amount: i64,
) -> TransactionResult {
    let vault_pda = accounts::get_vault_pda(test_config.node_operator.pubkey());
    let authority_config = accounts::get_authority_config_pda();
    let position_pda = accounts::get_position_pda(token_data.asset.pubkey());
    let inxs = UpdatePositionHandlerBuilder::new()
        .capital_provider(test_config.capital_provider.pubkey())
        .vault(vault_pda)
        .config(authority_config)
        .position(position_pda)
        .asset(token_data.asset.pubkey())
        .locking_token_mint(token_data.lock_mint)
        .vault_token_ata(token_data.vault_lock_ata)
        .capital_provider_token_ata(token_data.provider_lock_ata)
        .update_amount(update_amount)
        .instruction();
    utils::send_transaction(
        &mut test_config.svm,
        &[inxs],
        &test_config.god.pubkey(),
        &[
            &test_config.capital_provider.insecure_clone(),
            &test_config.god.insecure_clone(),
        ],
    )
}

pub fn capital_program_deposit_rewards(
    test_config: &mut TestConfig,
    token_data: &mut Tokens,
    reward_amount: u64,
) -> TransactionResult {
    let vault_pda = accounts::get_vault_pda(test_config.node_operator.pubkey());
    let authority_config = accounts::get_authority_config_pda();

    let inxs = DepositRewardsHandlerBuilder::new()
        .agent(test_config.agent.pubkey())
        .vault(vault_pda)
        .config(authority_config)
        .reward_token_mint(token_data.reward_mint)
        .vault_reward_ata(token_data.vault_reward_ata)
        .agent_reward_ata(token_data.agent_reward_ata)
        .amount(reward_amount)
        .instruction();
    utils::send_transaction(
        &mut test_config.svm,
        &[inxs],
        &test_config.god.pubkey(),
        &[
            &test_config.agent.insecure_clone(),
            &test_config.god.insecure_clone(),
        ],
    )
}

pub fn capital_program_claim_investor_rewards(
    test_config: &mut TestConfig,
    token_data: &mut Tokens,
) -> TransactionResult {
    let vault_pda = accounts::get_vault_pda(test_config.node_operator.pubkey());
    let authority_config = accounts::get_authority_config_pda();
    let position = accounts::get_position_pda(token_data.asset.pubkey());

    let inxs = ClaimInvestorRewardsHandlerBuilder::new()
        .holder(test_config.capital_provider.pubkey())
        .config(authority_config)
        .vault(vault_pda)
        .position(position)
        .asset(token_data.asset.pubkey())
        .reward_mint(token_data.reward_mint)
        .vault_ata(token_data.vault_reward_ata)
        .holder_ata(token_data.provider_lock_ata)
        .instruction();
    utils::send_transaction(
        &mut test_config.svm,
        &[inxs],
        &test_config.god.pubkey(),
        &[
            &test_config.capital_provider.insecure_clone(),
            &test_config.god.insecure_clone(),
        ],
    )
}
