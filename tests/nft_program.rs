mod setup;

use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::signature::Signer;
use zaals_finance_client::nft_program::accounts::NFTConfig;

use crate::setup::accounts;
#[test]
fn test_init_nft_program() {
    let mut test_config = TestConfig::new();
    let result = instruction_hadlers::init_nft_program(&mut test_config);

    match result {
        Ok(result) => {
            println!("Program logs is {:?}", result.logs);
            let nft_config = accounts::get_nft_config_pda();
            let nft_config_data: NFTConfig =
                accounts::get_data_from_pda_address(&mut test_config.svm, nft_config);
            assert_eq!(nft_config_data.admin, test_config.admin.pubkey());
            assert_eq!(
                nft_config_data.capital_program,
                test_config.capital_program_id
            );
        }
        Err(e) => panic!("Transaction failed: {:?}", e),
    }
}

#[test]
fn test_init_nft_program_twice_fails() {
    let mut test_config = TestConfig::new();
    let _ = instruction_hadlers::init_nft_program(&mut test_config);
    let result = instruction_hadlers::init_nft_program(&mut test_config);

    match result {
        Ok(_) => panic!("Transaction should have failed"),
        Err(e) => println!("Test succeded with failed initialization {:?}", e),
    }
}
