mod setup;

use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::signer::Signer;
use zaals_finance_client::capital_program::accounts::{AuthorityConfig, Vault};

use crate::setup::{
    accounts::{fund_ata, get_ata_balance, get_authority_config_pda, get_vault_pda},
    constants::{DEPOSIT_AMOUNT, MAX_VAULT_THRESHOLD, MIN_LOCK_AMOUNT, MIN_VAULT_TARGET},
    test_config::Tokens,
};

use utils::MplUtils;

#[test]
pub fn test_init_capital_program() {
    let mut test_config = TestConfig::new();
    let result = instruction_hadlers::init_capital_program(&mut test_config);

    match result {
        Ok(result) => {
            println!("instructions logs, {:?} ", result);
            let authority_config: AuthorityConfig = accounts::get_data_from_pda_address(
                &mut test_config.svm,
                get_authority_config_pda(),
            );
            assert_eq!(authority_config.agent, test_config.agent.pubkey());
        }
        Err(e) => {
            panic!("capital program initialization failed with {:?}", e);
        }
    }
}

#[test]

pub fn test_create_vault() {
    let mut test_config = TestConfig::new();
    let mut token_data = Tokens::create(&mut test_config);
    let _ = instruction_hadlers::init_nft_program(&mut test_config);
    let _ = instruction_hadlers::init_capital_program(&mut test_config);

    let result =
        instruction_hadlers::capital_program_create_vault(&mut test_config, &mut token_data);
    match result {
        Ok(result) => {
            for log in result.logs {
                println!("instruction log: {:?}", log);
            }
            let vault_config: Vault = accounts::get_data_from_pda_address(
                &mut test_config.svm,
                get_vault_pda(test_config.node_operator.pubkey()),
            );
            assert_eq!(
                vault_config.node_operator,
                test_config.node_operator.pubkey()
            );
            let mpl_collection =
                MplUtils::get_collection(&test_config.svm, &token_data.collection.pubkey());
            println!("MPL Collection: {:?}", mpl_collection);
            assert_eq!(vault_config.nft_collection, token_data.collection.pubkey());
            assert_eq!(vault_config.reward_token_mint, token_data.reward_mint);
            assert_eq!(vault_config.locking_token_mint, token_data.lock_mint);
            assert_eq!(vault_config.min_cap, MIN_VAULT_TARGET);
            assert_eq!(vault_config.min_lock_amount, MIN_LOCK_AMOUNT);
            assert_eq!(vault_config.max_cap, MAX_VAULT_THRESHOLD);
        }
        Err(e) => {
            panic!("instruction failed with {:?}", e);
        }
    }
}

#[test]

pub fn test_open_position() {
    let mut test_config = TestConfig::new();
    let mut token_data = Tokens::create(&mut test_config);

    instruction_hadlers::init_nft_program(&mut test_config).unwrap();
    instruction_hadlers::init_capital_program(&mut test_config).unwrap();
    instruction_hadlers::capital_program_create_vault(&mut test_config, &mut token_data).unwrap();
    let mpl_collection =
        MplUtils::get_collection(&test_config.svm, &token_data.collection.pubkey());
    println!("MPL Collection: {:?}", mpl_collection);

    let capital_provider = token_data.provider_lock_ata;
    let lock_mint = token_data.lock_mint;

    fund_ata(
        &mut test_config,
        &capital_provider,
        lock_mint,
        DEPOSIT_AMOUNT,
    );
    let result = instruction_hadlers::capital_program_open_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT,
    );
    match result {
        Ok(result) => {
            for log in result.logs {
                println!("instruction log: {:?}", log);
            }
            // let position_pda = accounts::get_position_pda(capital_provider);
        }

        Err(e) => {
            panic!("instruction failed with {:?}", e);
        }
    }
}
