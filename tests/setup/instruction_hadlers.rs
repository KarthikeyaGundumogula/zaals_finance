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
        CreateVaultHandlerBuilder, InitCapitalProgramHandlerBuilder, OpenPositionHandlerBuilder,
    },
    nft_program::instructions::InitNftProgramHandlerBuilder,
};

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

    // logs of all accounts involved in the transaction
    println!(
        "Capital Provider: {:?}",
        test_config.capital_provider.pubkey()
    );
    println!("Asset: {:?}", token_data.asset.pubkey());
    println!("Vault Collection: {:?}", token_data.collection.pubkey());
    println!("Vault PDA: {:?}", vault_pda);
    println!("Position PDA: {:?}", position_pda);
    println!("Provider Lock ATA: {:?}", token_data.provider_lock_ata);
    println!("Vault Lock ATA: {:?}", token_data.vault_lock_ata);
    println!("Locked Token Mint: {:?}", token_data.lock_mint);
    println!("Vault Lock ATA: {:?}", token_data.vault_lock_ata);

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
