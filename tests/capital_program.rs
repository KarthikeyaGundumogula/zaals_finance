mod setup;

use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::signer::Signer;
use zaals_finance_client::capital_program::accounts::{AuthorityConfig, Vault};

use crate::setup::{
    accounts::{get_authority_config_pda, get_position_vault_pda},
    constants::{MAX_VAULT_THRESHOLD, MIN_LOCK_AMOUNT, MIN_VAULT_TARGET},
    test_config::Tokens,
};

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
            println!("instructions logs, {:?} ", result);
            let vault_config: Vault = accounts::get_data_from_pda_address(
                &mut test_config.svm,
                get_position_vault_pda(test_config.node_operator.pubkey()),
            );
            assert_eq!(
                vault_config.node_operator,
                test_config.node_operator.pubkey()
            );
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
