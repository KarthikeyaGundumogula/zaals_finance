mod setup;

use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::signer::Signer;
use zaals_finance_client::capital_program::accounts::AuthorityConfig;

use crate::setup::capital_accounts::get_authority_config_pda;

#[test]
pub fn test_init_capital_program() {
    let mut test_config = TestConfig::new();
    let result = test_core_instructions::init_capital_program(&mut test_config);

    match result {
        Ok(result) => {
            println!("instructions logs, {:?} ", result);
            let authority_config:AuthorityConfig = capital_accounts::get_data_from_pda_address(&mut test_config.svm,get_authority_config_pda());
            assert_eq!(authority_config.agent,test_config.agent.pubkey());
        }
        Err(e) => {
            println!("capital program initialization failed with {:?}", e);
        }
    }
}