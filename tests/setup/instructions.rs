use crate::setup::{test_config::TestConfig, *};
use litesvm::types::TransactionResult;
use solana_sdk::signature::Signer;
use zaals_finance_client::{
    instructions::InitCapitalProgramHandlerBuilder,
    nft_program::instructions::InitNftProgramHandlerBuilder,
};

#[allow(dead_code)]
pub fn init_nft_program(test_config: &mut TestConfig) -> TransactionResult {
    let config_address = nft_accounts::get_nft_config_pda();
    let inxs = instructions::InitNftProgramHandlerBuilder::new()
        .admin(test_config.admin.pubkey())
        .authority(test_config.admin.pubkey())
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
            &test_config.admin.insecure_clone(),
        ],
    )
}

#[allow(dead_code)]
pub fn init_capital_program(test_config: &mut TestConfig) -> TransactionResult {
    let authority_config_address = capital_accounts::get_authority_config_pda();
    let nft_config_address = nft_accounts::get_nft_config_pda();
    let inxs = InitCapitalProgramHandlerBuilder::new()
        .admin(test_config.admin.pubkey())
        .agent(test_config.agent.pubkey())
        .config(authority_config_address)
        .nft_config(nft_config_address)
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
pub fn create_vault(test_config: &mut TestConfig) {}
