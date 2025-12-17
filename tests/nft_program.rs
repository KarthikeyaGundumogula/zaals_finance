use litesvm::{types::TransactionResult, LiteSVM};
use solana_sdk::{
    clock::Clock,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::read_keypair_file,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use zaals_finance_client::{
    accounts::NFTConfig, nft_program::instructions::InitializeProgramBuilder, CAPITAL_PROGRAM_ID,
    NFT_PROGRAM_ID,
};

#[test]
fn init_nft_program() {
    const NFT_PROGRAM_KEY_PAIR: &str = "target/deploy/nft_program-keypair.json";
    const NFT_PROGRAM_SO_FILE: &str = "target/deploy/nft_program.so";
    let capital_program_id = CAPITAL_PROGRAM_ID;
    let authority = Keypair::new();
    let mut svm = LiteSVM::new().with_sysvars();
    let admin = Keypair::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to fund payer");
    svm.airdrop(&admin.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to fund payer");
    svm.airdrop(&authority.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to fund payer");

    let program_keypair =
        read_keypair_file(NFT_PROGRAM_KEY_PAIR).expect("Failed to read keypair file");
    let program_id = program_keypair.pubkey();
    println!("Deploying program from keypair: {}", program_id);
    svm.add_program_from_file(program_id, NFT_PROGRAM_SO_FILE)
        .expect("Failed to deploy program from file");
    let config = Pubkey::try_find_program_address(&[b"nft_config"], &NFT_PROGRAM_ID);
    let config_address = config.unwrap().0;
    println!("config is {}", (config.unwrap()).0);
    let inxs = InitializeProgramBuilder::new()
        .admin(admin.pubkey())
        .authority(authority.pubkey())
        .capital_program(capital_program_id)
        .config(config_address)
        .instruction();
    let result = self::send_transaction(
        &mut svm,
        &[inxs],
        &payer.pubkey(),
        &[&payer.insecure_clone(), &admin.insecure_clone(), &authority],
    );

    match result {
        Ok(result) => {
            println!("Program logs is {:?}", result.logs);
            let account = svm
                .get_account(&config_address)
                .expect("Protocol config account not found");

            let nft_config = NFTConfig::from_bytes(&account.data).expect("Nft Config not found");
            println!("nft_config {:?}", nft_config);
        }
        Err(e) => panic!("Transaction failed: {:?}", e),
    }
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
