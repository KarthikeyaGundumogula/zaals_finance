#![allow(dead_code)]
use crate::constants::{MPL_CORE_ID, NFT_PROGRAM_KEY_PAIR, NFT_PROGRAM_SO_FILE};
use crate::setup::constants::{
    BASE_BPS, CAPITAL_PROGRAM_KEY_PAIR, CAPITAL_PROGRAM_SO_FILE, MPL_CORE_SO_FILE,
};
use litesvm::LiteSVM;
use litesvm::{error::LiteSVMError, types::TransactionResult};
use solana_sdk::{
    clock::Clock,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
use zaals_finance_client::capital_program::accounts::{Position, Vault};

use mpl_core::{Asset, Collection};

pub fn get_collection(svm: &LiteSVM, asset: &Pubkey) -> Collection {
    let account = svm
        .get_account(asset)
        .expect("Collection Asset account not found");

    *Collection::deserialize(&account.data).expect("Failed to deserialize collection account")
}

pub fn get_asset(svm: &LiteSVM, mint: &Pubkey) -> Asset {
    let account = svm.get_account(mint).expect("Asset account not found");

    *Asset::deserialize(&account.data).expect("Failed to deserialize asset account")
}

pub fn deploy_nft_program(svm: &mut LiteSVM) -> Result<(), LiteSVMError> {
    let program_keypair =
        read_keypair_file(NFT_PROGRAM_KEY_PAIR).expect("Failed to read keypair file");
    let program_id = program_keypair.pubkey();
    println!("Deploying program from keypair: {}", program_id);
    svm.add_program_from_file(program_id, NFT_PROGRAM_SO_FILE)
}

pub fn deploy_capital_program(svm: &mut LiteSVM) -> Result<(), LiteSVMError> {
    let program_keypair =
        read_keypair_file(CAPITAL_PROGRAM_KEY_PAIR).expect("Failed to read keypair file");
    let program_id = program_keypair.pubkey();
    println!("Deploying program from keypair: {}", program_id);
    svm.add_program_from_file(program_id, CAPITAL_PROGRAM_SO_FILE)
}

pub fn deploy_mpl_program(svm: &mut LiteSVM) -> Result<(), LiteSVMError> {
    let program_id = Pubkey::from_str(MPL_CORE_ID).expect("Invalid MPL_CORE_ID");
    svm.add_program_from_file(program_id, MPL_CORE_SO_FILE)
}

pub fn fund(svm: &mut LiteSVM, claimant: Pubkey) -> TransactionResult {
    svm.airdrop(&claimant, 100 * LAMPORTS_PER_SOL)
}

pub fn send_transaction(
    svm: &mut LiteSVM,
    instructions: &[Instruction],
    payer: &Pubkey,
    signing_keypairs: &[&Keypair],
) -> TransactionResult {
    let blockhash = svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(instructions, Some(payer), signing_keypairs, blockhash);
    let result = svm.send_transaction(tx);

    svm.expire_blockhash();
    let clock: Clock = svm.get_sysvar();
    svm.warp_to_slot(clock.slot + 100);

    result
}

pub fn set_clock(svm: &mut LiteSVM, unix_timestamp: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.slot = 10;
    clock.epoch = 1000;
    clock.unix_timestamp = unix_timestamp;
    svm.set_sysvar(&clock);
}

pub fn get_investor_accrued_rewards(vault: Vault, position: Position) -> u64 {
    let total_investor_bps: u16 = vault.investor_bps;

    // ---- step 1: total investor rewards (u128) ----
    let total_investors_rewards: u128 = (vault.total_rewards_deposited as u128)
        .checked_mul(total_investor_bps as u128)
        .expect("overflow in total_investors_rewards")
        .checked_div(BASE_BPS as u128)
        .expect("division by zero");

    // ---- step 2: position share (u128) ----
    let position_rewards: u128 = total_investors_rewards
        .checked_mul(position.total_value_locked as u128)
        .expect("overflow in position reward mul")
        .checked_div(vault.total_capital_collected as u128)
        .expect("division by zero");

    // ---- step 3: back to u64 ----
    position_rewards
        .checked_sub(position.total_rewards_claimed as u128)
        .expect("undefow in position rewards ")
        .try_into()
        .expect("position rewards exceed u64")
}

pub fn get_beneficiary_accrued_rewards(vault: Vault, beneficiary_index: usize) -> u64 {
    let beneficiary_bps: u16 = vault.beneficiaries[beneficiary_index].share_bps;

    let total_beneficiary_rewards: u128 = (vault.total_rewards_deposited as u128)
        .checked_mul(beneficiary_bps as u128)
        .expect("overflow in total_beneficiary_rewards")
        .checked_div(BASE_BPS as u128)
        .expect("division by zero");

    total_beneficiary_rewards
        .checked_sub(vault.beneficiaries[beneficiary_index].total_claimed as u128)
        .expect("underfow in beneficiary rewards")
        .try_into()
        .expect("beneficiary rewards exceed u64")
}

pub fn get_operator_accrued_rewards(vault: Vault) -> u64 {
    let beneficiaries_sum: u16 = vault.beneficiaries.iter().map(|b| b.share_bps).sum();
    let operator_bps = BASE_BPS - (beneficiaries_sum + vault.investor_bps);
    let total_operator_rewards = (vault.total_rewards_deposited as u128)
        .checked_mul(operator_bps as u128)
        .expect("overflow in total_beneficiary_rewards")
        .checked_div(BASE_BPS as u128)
        .expect("division by zero");
    total_operator_rewards
        .checked_sub(vault.operator_rewards_claimed as u128)
        .expect("underlow in operator rewards")
        .try_into()
        .expect("operator rewards exceeded u64")
}
