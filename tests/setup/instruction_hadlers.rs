#![allow(dead_code)]
#![allow(unused)]
use crate::setup::{
    accounts, constants::{
        FUND_RAISE_PERIOD, INVESTOR_BPS, MAX_SLASH_BPS, MAX_VAULT_THRESHOLD, MIN_LOCK_AMOUNT,
        MIN_VAULT_TARGET, ONE_DAY,
    },  test_config::{TestConfig, Tokens}, *
};
use litesvm::types::TransactionResult;
use solana_sdk::{clock::Clock, signature::Signer};

use zaals_finance_client::{
    capital_program::instructions::{CreateVaultHandlerBuilder, InitCapitalProgramHandlerBuilder},
    nft_program::instructions::InitNftProgramHandlerBuilder,
};

#[allow(dead_code)]
pub fn init_nft_program(test_config: &mut TestConfig) -> TransactionResult {
    let config_address = accounts::get_nft_config_pda();
    let authority_config = accounts::get_authority_config_pda();
    let inxs = instruction_hadlers::InitNftProgramHandlerBuilder::new()
        .admin(test_config.admin.pubkey())
        .authority(authority_config)
        .capital_program(test_config.capital_program_id)
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

#[allow(dead_code)]
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
        .nft_program(test_config.nft_program_id)
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

#[allow(dead_code)]
pub fn capital_program_create_vault(test_config: &mut TestConfig, token_data: &mut Tokens) -> TransactionResult {
    let position_vault =
        accounts::get_position_vault_pda(test_config.node_operator.pubkey());
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
        .min_cap(MIN_VAULT_TARGET)
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

#[allow(dead_code)]
pub fn capital_program_open_position(test_config: &mut TestConfig) {
    let vault_ata = accounts::get_position_vault_pda(test_config.capital_provider.pubkey());
    let authority_config = accounts::get_authority_config_pda();
    let nft_config = accounts::get_nft_config_pda();
}