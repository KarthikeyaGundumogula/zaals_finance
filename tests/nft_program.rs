mod setup;

use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::signature::Signer;


#[test]
fn test_init_nft_program() {
    let mut test_config = TestConfig::new();
    let result = test_core_instructions::init_nft_program(&mut test_config);

    match result {
        Ok(result) => {
            println!("Program logs is {:?}", result.logs);
            let nft_config_data = nft_accounts::get_nft_config_pda_data(&mut test_config.svm);
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
    let _ = test_core_instructions::init_nft_program(&mut test_config);
    let result = test_core_instructions::init_nft_program(&mut test_config);

    match result {
        Ok(_) => panic!("Transaction should have failed"),
        Err(e) => println!("Expected error: {:?}", e),
    }
}
