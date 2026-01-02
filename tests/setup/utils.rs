use std::str::FromStr;

use crate::constants::{NFT_PROGRAM_KEY_PAIR, NFT_PROGRAM_SO_FILE,MPL_CORE_ID};
use crate::setup::constants::{CAPITAL_PROGRAM_KEY_PAIR, CAPITAL_PROGRAM_SO_FILE, MPL_CORE_SO_FILE};
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

use mpl_core::{Asset, Collection};


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
pub struct MplUtils;

impl MplUtils {
    pub fn get_collection(svm: &LiteSVM, mint: &Pubkey) -> Collection {
        let account = svm
            .get_account(mint)
            .expect("Collection Asset account not found");

        *Collection::deserialize(&account.data).expect("Failed to deserialize collection account")
    }

    pub fn get_asset(svm: &LiteSVM, mint: &Pubkey) -> Asset {
        let account = svm.get_account(mint).expect("Asset account not found");

        *Asset::deserialize(&account.data).expect("Failed to deserialize asset account")
    }
}