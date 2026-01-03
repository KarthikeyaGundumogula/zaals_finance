mod setup;

use setup::test_config::TestConfig;
use setup::*;
use solana_sdk::signer::Signer;
use zaals_finance_client::capital_program::accounts::{AuthorityConfig, Position, Vault};

use crate::setup::{
    accounts::{
        fund_ata, get_ata_balance, get_authority_config_pda, get_data_from_pda_address,
        get_vault_pda,
    },
    constants::{DEPOSIT_AMOUNT, MAX_VAULT_THRESHOLD, MIN_LOCK_AMOUNT, ONE_DAY, REWARDS_DEPOSIT},
    test_config::Tokens,
    utils::set_clock,
};

#[test]
pub fn test_init_capital_program() {
    let mut test_config = TestConfig::new();
    let result = instruction_handlers::init_capital_program(&mut test_config);

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
    let _ = instruction_handlers::init_nft_program(&mut test_config);
    let _ = instruction_handlers::init_capital_program(&mut test_config);

    let result =
        instruction_handlers::capital_program_create_vault(&mut test_config, &mut token_data);
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
                utils::get_collection(&test_config.svm, &token_data.collection.pubkey());
            println!("MPL Collection: {:?}", mpl_collection);
            assert_eq!(vault_config.nft_collection, token_data.collection.pubkey());
            assert_eq!(vault_config.reward_token_mint, token_data.reward_mint);
            assert_eq!(vault_config.locking_token_mint, token_data.lock_mint);
            assert_eq!(vault_config.min_cap, MIN_LOCK_AMOUNT);
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
    let result = instruction_handlers::capital_program_open_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT,
    );
    match result {
        Ok(_) => {
            // for log in result.logs {
            //     println!("instruction log: {:?}", log);
            // }

            let vault_ata_balance =
                get_ata_balance(&mut test_config.svm, token_data.vault_lock_ata);
            let capital_provider_ata_balance =
                get_ata_balance(&mut test_config.svm, token_data.provider_lock_ata);
            let vault_pda = get_vault_pda(test_config.node_operator.pubkey());
            let position_pda = accounts::get_position_pda(token_data.asset.pubkey());
            let vault: Vault = accounts::get_data_from_pda_address(&mut test_config.svm, vault_pda);
            let asset_data = utils::get_asset(&test_config.svm, &token_data.asset.pubkey());
            let collection_data =
                utils::get_collection(&test_config.svm, &token_data.collection.pubkey());
            let position: Position =
                accounts::get_data_from_pda_address(&mut test_config.svm, position_pda);

            // ATA and ASSET ASSERTIONS
            assert_eq!(vault_ata_balance, DEPOSIT_AMOUNT);
            assert_eq!(
                asset_data.base.owner.to_string(),
                test_config.capital_provider.pubkey().to_string()
            );
            assert_eq!(collection_data.base.num_minted, 1);
            assert_eq!(capital_provider_ata_balance, 0);

            //VAULT DATA ASSERTIONS
            assert_eq!(vault.total_capital_collected, DEPOSIT_AMOUNT);
            assert_eq!(vault.capital_after_slashing, DEPOSIT_AMOUNT);

            // POSITION DATA ASSERTIONS
            assert_eq!(position.total_value_locked, DEPOSIT_AMOUNT);
            assert_eq!(position.asset, token_data.asset.pubkey());
        }

        Err(e) => {
            panic!("instruction failed with {:?}", e);
        }
    }
}

#[test]
pub fn test_increment_stake() {
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
        DEPOSIT_AMOUNT * 2,
    );
    instruction_handlers::capital_program_open_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT,
    )
    .unwrap();

    let result = instruction_handlers::capital_program_update_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT as i64,
    );
    match result {
        Ok(_) => {
            // for log in res.logs {
            //     println!("instruction log: {:?}", log);
            // }
            let vault_ata_balance =
                get_ata_balance(&mut test_config.svm, token_data.vault_lock_ata);
            let capital_provider_ata_balance =
                get_ata_balance(&mut test_config.svm, token_data.provider_lock_ata);
            let vault_pda = get_vault_pda(test_config.node_operator.pubkey());
            let position_pda = accounts::get_position_pda(token_data.asset.pubkey());
            let vault: Vault = accounts::get_data_from_pda_address(&mut test_config.svm, vault_pda);
            let position: Position =
                accounts::get_data_from_pda_address(&mut test_config.svm, position_pda);

            // ATA ASSERTIONS
            assert_eq!(vault_ata_balance, DEPOSIT_AMOUNT * 2);
            assert_eq!(capital_provider_ata_balance, 0);

            //VAULT DATA ASSERTIONS
            assert_eq!(vault.total_capital_collected, DEPOSIT_AMOUNT * 2);
            assert_eq!(vault.capital_after_slashing, DEPOSIT_AMOUNT * 2);

            // POSITION DATA ASSERTIONS
            assert_eq!(position.total_value_locked, DEPOSIT_AMOUNT * 2);
        }

        Err(e) => {
            panic!("instruction failed with {:?}", e);
        }
    }
}

#[test]
pub fn test_decrement_stake() {
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
        DEPOSIT_AMOUNT * 2,
    );
    instruction_handlers::capital_program_open_position(
        &mut test_config,
        &mut token_data,
        DEPOSIT_AMOUNT,
    )
    .unwrap();
    let result = instruction_handlers::capital_program_update_position(
        &mut test_config,
        &mut token_data,
        -(DEPOSIT_AMOUNT as i64 / 2),
    );

    match result {
        Ok(_) => {
            // for log in res.logs {
            //     println!("instruction log: {:?}", log);
            // }
            let vault_ata_balance =
                get_ata_balance(&mut test_config.svm, token_data.vault_lock_ata);
            let capital_provider_ata_balance =
                get_ata_balance(&mut test_config.svm, token_data.provider_lock_ata);
            let vault_pda = get_vault_pda(test_config.node_operator.pubkey());
            let position_pda = accounts::get_position_pda(token_data.asset.pubkey());
            let vault: Vault = accounts::get_data_from_pda_address(&mut test_config.svm, vault_pda);
            let position: Position =
                accounts::get_data_from_pda_address(&mut test_config.svm, position_pda);

            // ATA ASSERTIONS
            assert_eq!(vault_ata_balance, DEPOSIT_AMOUNT / 2);
            assert_eq!(
                capital_provider_ata_balance,
                (DEPOSIT_AMOUNT / 2) + DEPOSIT_AMOUNT
            );

            //VAULT DATA ASSERTIONS
            assert_eq!(vault.total_capital_collected, DEPOSIT_AMOUNT / 2);
            assert_eq!(vault.capital_after_slashing, DEPOSIT_AMOUNT / 2);

            // POSITION DATA ASSERTIONS
            assert_eq!(position.total_value_locked, DEPOSIT_AMOUNT / 2);
        }

        Err(e) => {
            panic!("instruction failed with {:?}", e);
        }
    }
}

#[test]
pub fn test_deposit_rewards() {
    let mut test_config = TestConfig::new();
    let mut token_data = Tokens::create(&mut test_config);

    instruction_handlers::init_nft_program(&mut test_config).unwrap();
    instruction_handlers::init_capital_program(&mut test_config).unwrap();
    instruction_handlers::capital_program_create_vault(&mut test_config, &mut token_data).unwrap();

    let capital_provider = token_data.provider_lock_ata;
    let lock_mint = token_data.lock_mint;
    let reward_mint = token_data.reward_mint;
    let agent_reward_ata = token_data.agent_reward_ata;

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

    fund_ata(
        &mut test_config,
        &agent_reward_ata,
        reward_mint,
        REWARDS_DEPOSIT,
    );

    let vault: Vault = accounts::get_data_from_pda_address(
        &mut test_config.svm,
        get_vault_pda(test_config.node_operator.pubkey()),
    );

    // TIME TRAVEL TO FUND RAISE PERIOD - GOO DORAEMON
    set_clock(&mut test_config.svm, vault.lock_phase_start_at + ONE_DAY);

    let result = instruction_handlers::capital_program_deposit_rewards(
        &mut test_config,
        &mut token_data,
        REWARDS_DEPOSIT,
    );

    match result {
        Ok(_) => {
            // for log in res.logs {
            //     println!("instruction log: {:?}", log);
            // }
            let vault_reward_ata_balance =
                get_ata_balance(&mut test_config.svm, token_data.vault_reward_ata);

            // ATA ASSERTIONS

            assert_eq!(vault_reward_ata_balance, REWARDS_DEPOSIT);

            // VAULT DATA ASSERTIONS
            let vault: Vault = accounts::get_data_from_pda_address(
                &mut test_config.svm,
                get_vault_pda(test_config.node_operator.pubkey()),
            );
            assert_eq!(vault.total_rewards_deposited, REWARDS_DEPOSIT);
        }

        Err(e) => {
            panic!("instruction failed with {:?}", e);
        }
    }
}

#[test]
pub fn test_claim_investor_rewards() {
    let mut test_config = TestConfig::new();
    let mut token_data = Tokens::create(&mut test_config);

    instruction_handlers::init_nft_program(&mut test_config).unwrap();
    instruction_handlers::init_capital_program(&mut test_config).unwrap();
    instruction_handlers::capital_program_create_vault(&mut test_config, &mut token_data).unwrap();

    let capital_provider = token_data.provider_lock_ata;
    let lock_mint = token_data.lock_mint;
    let reward_mint = token_data.reward_mint;
    let agent_reward_ata = token_data.agent_reward_ata;

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

    fund_ata(
        &mut test_config,
        &agent_reward_ata,
        reward_mint,
        REWARDS_DEPOSIT,
    );

    let vault: Vault = accounts::get_data_from_pda_address(
        &mut test_config.svm,
        get_vault_pda(test_config.node_operator.pubkey()),
    );

    // TIME TRAVEL TO FUND RAISE PERIOD - GOO DORAEMON
    set_clock(&mut test_config.svm, vault.lock_phase_start_at + ONE_DAY);

    instruction_handlers::capital_program_deposit_rewards(
        &mut test_config,
        &mut token_data,
        REWARDS_DEPOSIT,
    )
    .unwrap();

    let vault_pda = get_vault_pda(test_config.node_operator.pubkey());
    let position_pda = accounts::get_position_pda(token_data.asset.pubkey());
    let vault_data: Vault = get_data_from_pda_address(&mut test_config.svm, vault_pda);
    let position_data: Position = get_data_from_pda_address(&mut test_config.svm, position_pda);
    let holder_reward_ata_balance_before_claim =
        get_ata_balance(&mut test_config.svm, token_data.provider_reward_ata);
    let total_rewards_accrued = utils::get_investor_accrued_rewards(vault_data, position_data);
    println!(
        "Total rewards accrued before claim: {}",
        total_rewards_accrued
    );
    let result = instruction_handlers::capital_program_claim_investor_rewards(
        &mut test_config,
        &mut token_data,
    );

    match result {
        Ok(_) => {
            // for log in res.logs {
            //     println!("instruction log: {:?}", log);
            // }
            let holder_reward_ata_balance_after_claim =
                get_ata_balance(&mut test_config.svm, token_data.provider_reward_ata);
            assert_eq!(
                holder_reward_ata_balance_after_claim,
                holder_reward_ata_balance_before_claim + total_rewards_accrued
            );

            // ATA ASSERTIONS
        }

        Err(e) => {
            panic!("instruction failed with {:?}", e);
        }
    }
}
