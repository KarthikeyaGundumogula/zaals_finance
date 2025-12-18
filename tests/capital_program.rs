mod setup;

use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::signer::Signer;

#[test]
pub fn test_init_capital_program() {
    let mut test_config = TestConfig::new();
    let result = instructions::init_capital_program(&mut test_config);

    match result {
        Ok(result) => {
            println!("instructions logs, {:?} ", result);
            let authority_config = capital_accounts::get_authority_config_pda_data(&mut test_config.svm);
            assert_eq!(authority_config.agent,test_config.agent.pubkey());
        }
        Err(e) => {
            println!("capital program initialization failed with {:?}", e);
        }
    }
}