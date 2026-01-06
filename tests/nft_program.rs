mod setup;
use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::clock::Clock;
use solana_sdk::signature::Signer;
use zaals_finance_client::nft_program::accounts::{NFTConfig, Offer};
use zaals_finance_client::CAPITAL_PROGRAM_ID;

use crate::setup::accounts::{self, fund_ata};
use crate::setup::constants::{DEPOSIT_AMOUNT, ONE_DAY, SALE_PRICE};
use crate::setup::test_config::Tokens;
use crate::setup::utils::set_clock;
#[test]
fn test_init_nft_program() {
    let mut test_config = TestConfig::new();
    let result = instruction_handlers::init_nft_program(&mut test_config);

    match result {
        Ok(result) => {
            println!("Program logs is {:?}", result.logs);
            let nft_config = accounts::get_nft_config_pda();
            let nft_config_data: NFTConfig =
                accounts::get_data_from_pda_address(&mut test_config.svm, nft_config);
            assert_eq!(nft_config_data.admin, test_config.admin.pubkey());
            assert_eq!(nft_config_data.capital_program, CAPITAL_PROGRAM_ID);
        }
        Err(e) => panic!("Transaction failed: {:?}", e),
    }
}

#[test]
fn test_list_position() {
    let mut test_config = TestConfig::new();
    let mut token_data = Tokens::create(&mut test_config);

    instruction_handlers::init_nft_program(&mut test_config).unwrap();
    instruction_handlers::init_capital_program(&mut test_config).unwrap();
    instruction_handlers::capital_program_create_vault(&mut test_config, &mut token_data).unwrap();

    let capital_provider = token_data.provider_lock_ata;
    let lock_mint = token_data.lock_mint;

    fund_ata(
        &mut test_config,
        &capital_provider,
        lock_mint,
        DEPOSIT_AMOUNT,
    );
    instruction_handlers::capital_program_open_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT,
    )
    .unwrap();

    let clock: Clock = test_config.svm.get_sysvar();
    set_clock(&mut test_config.svm, clock.unix_timestamp + ONE_DAY);

    let result = instruction_handlers::nft_program_list_positon(&mut test_config, &mut token_data);
    match result {
        Ok(_) => {
            // for log in r.logs {
            //     println!("{log}");
            // }
            let offer_pda = accounts::get_offer_pda(token_data.asset.pubkey());
            let asset = utils::get_asset(&test_config.svm, &token_data.asset.pubkey());
            assert_eq!(asset.base.owner.to_string(), offer_pda.to_string());
            let offer_data: Offer =
                accounts::get_data_from_pda_address(&mut test_config.svm, offer_pda);
            assert_eq!(offer_data.price, SALE_PRICE);
            assert_eq!(offer_data.token_mint, token_data.general_mint);
        }
        Err(e) => {
            panic!("instruction failed with{:?} ", e)
        }
    }
}

#[test]
fn test_unlist_position() {
    let mut test_config = TestConfig::new();
    let mut token_data = Tokens::create(&mut test_config);

    instruction_handlers::init_nft_program(&mut test_config).unwrap();
    instruction_handlers::init_capital_program(&mut test_config).unwrap();
    instruction_handlers::capital_program_create_vault(&mut test_config, &mut token_data).unwrap();

    let capital_provider = token_data.provider_lock_ata;
    let lock_mint = token_data.lock_mint;

    fund_ata(
        &mut test_config,
        &capital_provider,
        lock_mint,
        DEPOSIT_AMOUNT,
    );
    instruction_handlers::capital_program_open_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT,
    )
    .unwrap();

    let clock: Clock = test_config.svm.get_sysvar();
    set_clock(&mut test_config.svm, clock.unix_timestamp + ONE_DAY);

    instruction_handlers::nft_program_list_positon(&mut test_config, &mut token_data).unwrap();

    set_clock(&mut test_config.svm, clock.unix_timestamp + 3 * ONE_DAY);
    let result =
        instruction_handlers::nft_program_unlist_positon(&mut test_config, &mut token_data);
    match result {
        Ok(res) => {
            for log in res.logs {
                println!("{log}");
            }
            let asset = utils::get_asset(&test_config.svm, &token_data.asset.pubkey());
            assert_eq!(
                asset.base.owner.to_string(),
                test_config.capital_provider.pubkey().to_string()
            );
        }
        Err(e) => {
            panic!("Instruction failed with {e:?}");
        }
    }
}

#[test]
fn test_buy_position_handler() {
    let mut test_config = TestConfig::new();
    let mut token_data = Tokens::create(&mut test_config);

    instruction_handlers::init_nft_program(&mut test_config).unwrap();
    instruction_handlers::init_capital_program(&mut test_config).unwrap();
    instruction_handlers::capital_program_create_vault(&mut test_config, &mut token_data).unwrap();

    let capital_provider = token_data.provider_lock_ata;
    let lock_mint = token_data.lock_mint;

    fund_ata(
        &mut test_config,
        &capital_provider,
        lock_mint,
        DEPOSIT_AMOUNT,
    );
    instruction_handlers::capital_program_open_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT,
    )
    .unwrap();

    let clock: Clock = test_config.svm.get_sysvar();
    set_clock(&mut test_config.svm, clock.unix_timestamp + ONE_DAY);

    instruction_handlers::nft_program_list_positon(&mut test_config, &mut token_data).unwrap();
    fund_ata(
        &mut test_config,
        &token_data.admin_general_ata,
        token_data.general_mint,
        SALE_PRICE * 2,
    );
    let result = instruction_handlers::nft_program_buy_position(&mut test_config, &mut token_data);
    set_clock(&mut test_config.svm, clock.unix_timestamp + 2 * ONE_DAY);
    match result {
        Ok(_) => {
            // for log in r.logs {
            //     println!("{log}");
            // }
            let asset = utils::get_asset(&test_config.svm, &token_data.asset.pubkey());
            assert_eq!(
                asset.base.owner.to_string(),
                test_config.admin.pubkey().to_string()
            );
        }
        Err(e) => {
            panic!("instruction failed with{:?} ", e)
        }
    }
}

#[test]
fn test_init_nft_program_twice_fails() {
    let mut test_config = TestConfig::new();
    let _ = instruction_handlers::init_nft_program(&mut test_config);
    let result = instruction_handlers::init_nft_program(&mut test_config);

    match result {
        Ok(_) => panic!("Transaction should have failed"),
        Err(e) => println!("Test succeded with failed initialization {:?}", e),
    }
}
